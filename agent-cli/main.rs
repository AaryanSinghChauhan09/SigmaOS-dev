// SPDX-License-Identifier: Apache-2.0
//! SigmaOS Agent CLI Dispatcher & Runtime Manager
//! Unified interface for universal coding agents, usage tracking, local LLMs, and crash diagnostics.

#![forbid(unsafe_code)]

use std::env;
use std::path::Path;
use std::process::{Command, ExitCode};

fn print_banner() {
    println!("╔════════════════════════════════════════════╗");
    println!("║      SigmaOS AI & Agentic Suite (Omarchy 4+)║");
    println!("╚════════════════════════════════════════════╝");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let subcmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match subcmd {
        "list" | "agents" => {
            print_banner();
            println!("Pre-wired lazy-loaded coding agents:");
            println!("  • claude    Anthropic Claude Code CLI");
            println!("  • codex     OpenAI Codex CLI");
            println!("  • opencode  OpenCode terminal agent");
            println!("  • agy       Antigravity CLI (Google DeepMind)");
            println!("  • copilot   GitHub Copilot CLI");
            println!("  • crush     Crush agent");
            println!("  • grok      xAI Grok terminal agent");
            println!("  • pi        Inflection Pi CLI");
            println!("  • omp       Oh My Posh / Agent helper");
            println!("  • ori       Ori agent runtime");
            ExitCode::SUCCESS
        }
        "usage" => {
            let script = Path::new("bin/sigmaos-agent-usage");
            if script.exists() {
                let status = Command::new("bash").arg(script).args(&args[2..]).status();
                match status {
                    Ok(s) if s.success() => ExitCode::SUCCESS,
                    _ => ExitCode::FAILURE,
                }
            } else {
                eprintln!("[!] bin/sigmaos-agent-usage not found");
                ExitCode::FAILURE
            }
        }
        "crash-diag" | "crash" => {
            let script = Path::new("bin/sigmaos-crash-diagnostics");
            if script.exists() {
                let status = Command::new("bash").arg(script).args(&args[2..]).status();
                match status {
                    Ok(s) if s.success() => ExitCode::SUCCESS,
                    _ => ExitCode::FAILURE,
                }
            } else {
                eprintln!("[!] bin/sigmaos-crash-diagnostics not found");
                ExitCode::FAILURE
            }
        }
        "local-ai" | "ollama" => {
            let script = Path::new("bin/sigmaos-local-ai");
            if script.exists() {
                let status = Command::new("bash").arg(script).args(&args[2..]).status();
                match status {
                    Ok(s) if s.success() => ExitCode::SUCCESS,
                    _ => ExitCode::FAILURE,
                }
            } else {
                eprintln!("[!] bin/sigmaos-local-ai not found");
                ExitCode::FAILURE
            }
        }
        "run" | "launch" => {
            let script = Path::new("bin/sigmaos-agent");
            if script.exists() {
                let status = Command::new("bash").arg(script).args(&args[2..]).status();
                match status {
                    Ok(s) if s.success() => ExitCode::SUCCESS,
                    _ => ExitCode::FAILURE,
                }
            } else {
                eprintln!("[!] bin/sigmaos-agent not found");
                ExitCode::FAILURE
            }
        }
        _ => {
            print_banner();
            println!("Usage: sigmaos_agent_cli <command> [args...]");
            println!("");
            println!("Commands:");
            println!("  agents, list    List all supported AI coding agents");
            println!("  run <agent>     Launch or switch active coding agent");
            println!("  usage           Display top-bar AI token usage and subscription limits");
            println!("  crash-diag      Inspect systemd-coredump crash logs and route to agent");
            println!("  local-ai        Manage local LLMs (Ollama / LM Studio GPU inference)");
            ExitCode::SUCCESS
        }
    }
}
