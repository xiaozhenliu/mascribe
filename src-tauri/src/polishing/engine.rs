use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;
use std::path::Path;

const MAX_GENERATION_TOKENS: usize = 512;

pub struct PolishingEngine {
    backend: LlamaBackend,
    model: LlamaModel,
}

impl PolishingEngine {
    /// Load a GGUF model for text polishing.
    /// `n_gpu_layers`: number of layers to offload to GPU (99 = all layers on Apple Silicon).
    pub fn new(model_path: &str, n_gpu_layers: u32) -> anyhow::Result<Self> {
        let path = Path::new(model_path);
        if !path.exists() {
            anyhow::bail!("Polishing model not found: {}", path.display());
        }

        let backend = LlamaBackend::init()
            .map_err(|e| anyhow::anyhow!("Failed to init llama backend: {}", e))?;

        let model_params = LlamaModelParams::default().with_n_gpu_layers(n_gpu_layers);

        let model = LlamaModel::load_from_file(&backend, path, &model_params)
            .map_err(|e| anyhow::anyhow!("Failed to load polishing model: {}", e))?;

        println!(
            "[PolishingEngine] Model loaded from: {} (vocab={})",
            model_path,
            model.n_vocab()
        );

        Ok(Self { backend, model })
    }

    /// Polish transcribed text by fixing punctuation, grammar, and capitalization.
    /// Returns the polished text, or the original text if polishing fails.
    pub fn polish(&self, text: &str) -> anyhow::Result<String> {
        if text.trim().is_empty() {
            return Ok(text.to_string());
        }

        let prompt = build_prompt(text);

        // Create a fresh context for each inference call.
        // Context creation is fast (~2ms) and avoids KV cache management complexity.
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(1024))
            .with_n_threads(4)
            .with_n_threads_batch(4)
            .with_n_batch(512);

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| anyhow::anyhow!("Failed to create context: {}", e))?;

        // Tokenize the prompt
        let tokens = self
            .model
            .str_to_token(&prompt, AddBos::Always)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;

        println!(
            "[polish] prompt: {} tokens, input: '{}' ({} chars)",
            tokens.len(),
            text,
            text.len()
        );

        // Feed prompt tokens into batch
        let mut batch = LlamaBatch::new(1024, 1);
        batch
            .add_sequence(&tokens, 0, false)
            .map_err(|e| anyhow::anyhow!("Batch add failed: {}", e))?;

        // Enable logits for the last token only (needed for sampling)
        // add_sequence with logits_all=false already does this

        // Decode the prompt (prefill)
        ctx.decode(&mut batch)
            .map_err(|e| anyhow::anyhow!("Prompt decode failed: {}", e))?;

        // Setup sampler: low temperature (near-greedy) for deterministic, faithful output
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(0.1),
            LlamaSampler::greedy(),
        ]);

        // Generation loop
        let mut output = String::new();
        let mut n_decoded = tokens.len();
        let mut decoder = encoding_rs::UTF_8.new_decoder();

        for _ in 0..MAX_GENERATION_TOKENS {
            // Sample next token
            let token = sampler.sample(&ctx, -1);

            // Check for end of generation
            if self.model.is_eog_token(token) {
                break;
            }

            // Convert token to string piece
            match self.model.token_to_piece(token, &mut decoder, true, None) {
                Ok(piece) => output.push_str(&piece),
                Err(e) => {
                    println!("[polish] token_to_piece error: {}, stopping", e);
                    break;
                }
            }

            // Prepare next decode step
            batch.clear();
            batch
                .add(token, n_decoded as i32, &[0], true)
                .map_err(|e| anyhow::anyhow!("Batch add failed: {}", e))?;
            n_decoded += 1;

            ctx.decode(&mut batch)
                .map_err(|e| anyhow::anyhow!("Decode failed: {}", e))?;
        }

        Ok(validate_output(text, &output))
    }
}

/// Build the Gemma 3 chat-format prompt for text polishing.
fn build_prompt(text: &str) -> String {
    format!(
        "<start_of_turn>user\n\
         You are a text post-processor for speech-to-text output. \
         Fix only punctuation, capitalization, and grammar. \
         Do not change the meaning, do not add or remove content, do not translate. \
         Keep the original language. If the text is already correct, return it unchanged.\n\
         \n\
         Text to fix:\n\
         {}<end_of_turn>\n\
         <start_of_turn>model\n",
        text
    )
}

/// Validate LLM output: reject empty or suspiciously long outputs.
fn validate_output(original: &str, generated: &str) -> String {
    let trimmed = generated.trim().to_string();
    if trimmed.is_empty() {
        println!("[polish] empty output, using original");
        return original.to_string();
    }
    // Reject if output is suspiciously longer than input (likely hallucination)
    if trimmed.len() > original.len() * 2 + 50 {
        println!(
            "[polish] output too long ({} vs {} chars), using original",
            trimmed.len(),
            original.len()
        );
        return original.to_string();
    }
    trimmed
}
