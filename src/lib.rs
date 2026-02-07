use zed_extension_api::{self as zed, Result, SlashCommand, SlashCommandOutput, SlashCommandOutputSection};

mod validator;

use validator::InputValidator;

struct MermaidExtension {
    validator: InputValidator,
}

impl zed::Extension for MermaidExtension {
    fn new() -> Self {
        Self {
            validator: InputValidator::new(),
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<SlashCommandOutput> {
        match command.name.as_str() {
            "mermaid-preview" => {
                // TODO: Implement preview functionality
                let text = "Mermaid preview will be implemented here".to_string();

                Ok(SlashCommandOutput {
                    text: text.clone(),
                    sections: vec![SlashCommandOutputSection {
                        range: (0..text.len()).into(),
                        label: "Mermaid Preview".to_string(),
                    }],
                })
            }
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }
}

zed::register_extension!(MermaidExtension);
