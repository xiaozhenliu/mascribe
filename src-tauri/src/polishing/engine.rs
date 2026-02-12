use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;
use std::path::Path;

const MAX_GENERATION_TOKENS: usize = 512;

/// Chat template format for different model families.
#[derive(Debug, Clone, Copy)]
enum ChatFormat {
    /// Gemma 3: <start_of_turn>user\n...<end_of_turn>\n<start_of_turn>model\n
    Gemma,
    /// ChatML (Qwen, Yi, etc.): <|im_start|>user\n...<|im_end|>\n<|im_start|>assistant\n
    ChatML,
}

pub struct PolishingEngine {
    backend: LlamaBackend,
    model: LlamaModel,
    chat_format: ChatFormat,
}

impl PolishingEngine {
    /// Load a GGUF model for text polishing.
    /// `n_gpu_layers`: number of layers to offload to GPU (99 = all layers on Apple Silicon).
    /// Auto-detects chat format from model filename.
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

        // Auto-detect chat format from filename
        let lower_path = model_path.to_lowercase();
        let chat_format = if lower_path.contains("qwen") || lower_path.contains("yi-") {
            ChatFormat::ChatML
        } else {
            ChatFormat::Gemma
        };

        println!(
            "[PolishingEngine] Model loaded: {} (vocab={}, format={:?})",
            model_path,
            model.n_vocab(),
            chat_format
        );

        Ok(Self {
            backend,
            model,
            chat_format,
        })
    }

    /// Polish transcribed text by fixing punctuation, grammar, and capitalization.
    /// If `custom_prompt` is provided, it should contain `{text}` as a placeholder.
    /// `detected_lang` is the language code from SenseVoice (e.g. "zh", "en", "ja").
    pub fn polish(
        &self,
        text: &str,
        custom_prompt: Option<&str>,
        detected_lang: Option<&str>,
    ) -> anyhow::Result<String> {
        if text.trim().is_empty() {
            return Ok(text.to_string());
        }

        let prompt = build_prompt(text, custom_prompt, detected_lang, self.chat_format);

        // Create a fresh context for each inference call.
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(2048))
            .with_n_threads(4)
            .with_n_threads_batch(4)
            .with_n_batch(2048);

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
            "[polish:local] {} tokens, input: '{}' ({} chars, {:?})",
            tokens.len(),
            text,
            text.len(),
            self.chat_format
        );

        // Feed prompt tokens into batch
        let mut batch = LlamaBatch::new(2048, 1);
        batch
            .add_sequence(&tokens, 0, false)
            .map_err(|e| anyhow::anyhow!("Batch add failed: {}", e))?;

        // Decode the prompt (prefill)
        ctx.decode(&mut batch)
            .map_err(|e| anyhow::anyhow!("Prompt decode failed: {}", e))?;

        // Setup sampler: low temperature for deterministic output
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(0.15),
            LlamaSampler::greedy(),
        ]);

        // Generation loop
        let mut output = String::new();
        let mut n_decoded = tokens.len();
        let mut decoder = encoding_rs::UTF_8.new_decoder();

        for _ in 0..MAX_GENERATION_TOKENS {
            let token = sampler.sample(&ctx, -1);

            if self.model.is_eog_token(token) {
                break;
            }

            match self.model.token_to_piece(token, &mut decoder, true, None) {
                Ok(piece) => output.push_str(&piece),
                Err(e) => {
                    println!("[polish:local] token_to_piece error: {}, stopping", e);
                    break;
                }
            }

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

/// Build a chat-format prompt for text polishing.
/// Selects the correct template format based on the model family.
fn build_prompt(
    text: &str,
    custom_prompt: Option<&str>,
    detected_lang: Option<&str>,
    format: ChatFormat,
) -> String {
    let lang = detected_lang.unwrap_or("auto");
    let user_content = match custom_prompt {
        Some(template) => template.replace("{text}", text).replace("{lang}", lang),
        None => format!(
            "Language: {}\n\
             Clean up this speech transcript. Output ONLY the cleaned text.\n\
             - Remove filler words\n\
             - Fix punctuation\n\
             - Do NOT translate\n\n\
             {}",
            lang, text
        ),
    };
    match format {
        ChatFormat::Gemma => format!(
            "<start_of_turn>user\n{}<end_of_turn>\n<start_of_turn>model\n",
            user_content
        ),
        ChatFormat::ChatML => format!(
            "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
            user_content
        ),
    }
}

/// Count how many characters in a string are CJK (Chinese/Japanese/Korean).
fn cjk_char_count(s: &str) -> usize {
    s.chars()
        .filter(|c| {
            let cp = *c as u32;
            (0x4E00..=0x9FFF).contains(&cp)
                || (0x3400..=0x4DBF).contains(&cp)
                || (0x3000..=0x303F).contains(&cp)
                || (0xFF00..=0xFFEF).contains(&cp)
        })
        .count()
}

/// Check if text is predominantly CJK (>30% of non-whitespace chars).
fn is_cjk_text(s: &str) -> bool {
    let non_ws: usize = s.chars().filter(|c| !c.is_whitespace()).count();
    if non_ws == 0 {
        return false;
    }
    let cjk = cjk_char_count(s);
    (cjk as f64 / non_ws as f64) > 0.3
}

/// Validate LLM output: reject clearly broken outputs, but allow polishing to work.
fn validate_output(original: &str, generated: &str) -> String {
    let trimmed = generated.trim().to_string();
    if trimmed.is_empty() {
        println!("[polish:local] empty output, using original");
        return original.to_string();
    }
    if trimmed.len() > original.len() * 3 + 100 {
        println!(
            "[polish:local] output too long ({} vs {} chars), using original",
            trimmed.len(),
            original.len()
        );
        return original.to_string();
    }
    if is_cjk_text(original) && !is_cjk_text(&trimmed) {
        println!("[polish:local] language switch detected (CJK→non-CJK), using original");
        return original.to_string();
    }
    // Strip common LLM meta-commentary prefixes
    let cleaned = strip_meta_prefix(&trimmed);
    if cleaned.is_empty() {
        println!("[polish:local] only meta-commentary, using original");
        return original.to_string();
    }
    cleaned
}

/// Strip common LLM meta-commentary prefixes like "Here is the cleaned text:\n"
fn strip_meta_prefix(s: &str) -> String {
    let lower = s.to_lowercase();
    let prefixes = [
        "here is the cleaned text:",
        "here's the cleaned text:",
        "here is the corrected text:",
        "here's the corrected text:",
        "okay, here",
        "sure, here",
        "corrected text:",
        "cleaned text:",
    ];
    for prefix in &prefixes {
        if lower.starts_with(prefix) {
            let rest = s[prefix.len()..].trim();
            if !rest.is_empty() {
                println!("[polish:local] stripped meta-prefix: '{}'", prefix);
                return rest.to_string();
            }
        }
    }
    if (lower.starts_with("here") || lower.starts_with("okay") || lower.starts_with("sure"))
        && s.contains('\n')
    {
        if let Some(idx) = s.find('\n') {
            let rest = s[idx + 1..].trim();
            if !rest.is_empty() {
                println!("[polish:local] stripped first-line meta: '{}'", &s[..idx]);
                return rest.to_string();
            }
        }
    }
    s.to_string()
}
