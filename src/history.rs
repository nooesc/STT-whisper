use std::fs;
use std::path::Path;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandEntry {
    pub timestamp: DateTime<Local>,
    pub transcription: String,
    pub command_matched: Option<String>,
    pub command_executed: Option<String>,
    pub success: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandHistory {
    entries: Vec<CommandEntry>,
}

impl CommandHistory {
    pub fn new() -> Self {
        CommandHistory {
            entries: Vec::new(),
        }
    }

    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !Path::new(path).exists() {
            return Ok(Self::new());
        }
        
        let contents = fs::read_to_string(path)?;
        let history: CommandHistory = serde_json::from_str(&contents)?;
        Ok(history)
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn add_entry(&mut self, entry: CommandEntry) {
        self.entries.push(entry);
        
        // Keep only the last 1000 entries
        if self.entries.len() > 1000 {
            self.entries.drain(0..self.entries.len() - 1000);
        }
    }

    pub fn get_statistics(&self) -> CommandStatistics {
        let total_commands = self.entries.len();
        let successful_commands = self.entries.iter().filter(|e| e.success).count();
        let failed_commands = total_commands - successful_commands;
        
        let mut command_usage = std::collections::HashMap::new();
        for entry in &self.entries {
            if let Some(cmd) = &entry.command_matched {
                *command_usage.entry(cmd.clone()).or_insert(0) += 1;
            }
        }
        
        let avg_duration = if total_commands > 0 {
            self.entries.iter().map(|e| e.duration_ms).sum::<u64>() / total_commands as u64
        } else {
            0
        };

        CommandStatistics {
            total_commands,
            successful_commands,
            failed_commands,
            command_usage,
            avg_duration_ms: avg_duration,
        }
    }

    pub fn get_recent_entries(&self, count: usize) -> Vec<&CommandEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    pub fn get_all_entries(&self) -> &Vec<CommandEntry> {
        &self.entries
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandStatistics {
    pub total_commands: usize,
    pub successful_commands: usize,
    pub failed_commands: usize,
    pub command_usage: std::collections::HashMap<String, usize>,
    pub avg_duration_ms: u64,
}

impl CommandStatistics {
    pub fn print_summary(&self) {
        println!("\n📊 Command Statistics:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Total commands: {}", self.total_commands);
        println!("Successful: {} ({:.1}%)", 
            self.successful_commands, 
            if self.total_commands > 0 {
                (self.successful_commands as f64 / self.total_commands as f64) * 100.0
            } else {
                0.0
            }
        );
        println!("Failed: {}", self.failed_commands);
        println!("Average duration: {}ms", self.avg_duration_ms);
        
        if !self.command_usage.is_empty() {
            println!("\n🔥 Most used commands:");
            let mut usage_vec: Vec<_> = self.command_usage.iter().collect();
            usage_vec.sort_by(|a, b| b.1.cmp(a.1));
            
            for (i, (cmd, count)) in usage_vec.iter().take(5).enumerate() {
                println!("  {}. {} ({} times)", i + 1, cmd, count);
            }
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}