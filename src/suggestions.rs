use std::collections::HashMap;
use chrono::{Local, Timelike, Datelike};
use crate::history::CommandHistory;

pub struct SmartSuggestions {
    min_confidence: f32,
}

impl SmartSuggestions {
    pub fn new() -> Self {
        SmartSuggestions {
            min_confidence: 0.7,
        }
    }

    pub fn fuzzy_match(&self, input: &str, target: &str) -> f32 {
        let input_lower = input.to_lowercase();
        let target_lower = target.to_lowercase();
        
        if input_lower == target_lower {
            return 1.0;
        }
        
        if target_lower.contains(&input_lower) || input_lower.contains(&target_lower) {
            return 0.8;
        }
        
        self.levenshtein_similarity(&input_lower, &target_lower)
    }

    fn levenshtein_similarity(&self, s1: &str, s2: &str) -> f32 {
        let len1 = s1.len();
        let len2 = s2.len();
        let max_len = len1.max(len2);
        
        if max_len == 0 {
            return 1.0;
        }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        
        1.0 - (matrix[len1][len2] as f32 / max_len as f32)
    }

    pub fn find_best_match<'a>(&self, input: &str, commands: &'a HashMap<String, String>) -> Option<(&'a str, f32)> {
        let mut best_match = None;
        let mut best_score = 0.0;
        
        for (phrase, _) in commands {
            let score = self.fuzzy_match(input, phrase);
            if score > best_score && score >= self.min_confidence {
                best_score = score;
                best_match = Some(phrase.as_str());
            }
        }
        
        best_match.map(|m| (m, best_score))
    }

    pub fn get_time_based_suggestions(&self, history: &CommandHistory, limit: usize) -> Vec<String> {
        let now = Local::now();
        let current_hour = now.hour();
        let current_day = now.weekday();
        
        let entries = history.get_all_entries();
        let mut time_patterns: HashMap<String, TimePattern> = HashMap::new();
        
        for entry in entries {
            if let Some(cmd) = &entry.command_matched {
                if entry.success {
                    let entry_hour = entry.timestamp.hour();
                    let entry_day = entry.timestamp.weekday();
                    
                    let pattern = time_patterns.entry(cmd.clone()).or_insert(TimePattern::new());
                    pattern.add_occurrence(entry_hour, entry_day);
                }
            }
        }
        
        let mut suggestions: Vec<(String, f32)> = time_patterns
            .into_iter()
            .map(|(cmd, pattern)| {
                let relevance = pattern.calculate_relevance(current_hour, current_day);
                (cmd, relevance)
            })
            .filter(|(_, relevance)| *relevance > 0.3)
            .collect();
        
        suggestions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        suggestions
            .into_iter()
            .take(limit)
            .map(|(cmd, _)| cmd)
            .collect()
    }

    pub fn get_frequency_suggestions(&self, history: &CommandHistory, limit: usize) -> Vec<String> {
        let stats = history.get_statistics();
        let mut usage_vec: Vec<_> = stats.command_usage.into_iter().collect();
        usage_vec.sort_by(|a, b| b.1.cmp(&a.1));
        
        usage_vec
            .into_iter()
            .take(limit)
            .map(|(cmd, _)| cmd)
            .collect()
    }

    pub fn get_suggestions_for_failed_command(&self, input: &str, history: &CommandHistory, commands: &HashMap<String, String>) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if let Some((best_match, score)) = self.find_best_match(input, commands) {
            suggestions.push(format!("Did you mean: {} ({}% match)?", best_match, (score * 100.0) as i32));
        }
        
        let time_suggestions = self.get_time_based_suggestions(history, 2);
        if !time_suggestions.is_empty() {
            suggestions.push(format!("Based on your usage patterns, try: {}", time_suggestions.join(" or ")));
        }
        
        suggestions
    }
}

struct TimePattern {
    hour_counts: [u32; 24],
    day_counts: [u32; 7],
    total_count: u32,
}

impl TimePattern {
    fn new() -> Self {
        TimePattern {
            hour_counts: [0; 24],
            day_counts: [0; 7],
            total_count: 0,
        }
    }

    fn add_occurrence(&mut self, hour: u32, day: chrono::Weekday) {
        self.hour_counts[hour as usize] += 1;
        self.day_counts[day.num_days_from_monday() as usize] += 1;
        self.total_count += 1;
    }

    fn calculate_relevance(&self, current_hour: u32, current_day: chrono::Weekday) -> f32 {
        if self.total_count == 0 {
            return 0.0;
        }
        
        let hour_relevance = self.calculate_hour_relevance(current_hour);
        let day_relevance = self.day_counts[current_day.num_days_from_monday() as usize] as f32 / self.total_count as f32;
        
        hour_relevance * 0.7 + day_relevance * 0.3
    }

    fn calculate_hour_relevance(&self, current_hour: u32) -> f32 {
        let mut relevance = 0.0;
        let window = 2;
        
        for offset in 0..=window {
            let hour_before = ((current_hour as i32 - offset as i32 + 24) % 24) as usize;
            let hour_after = ((current_hour + offset) % 24) as usize;
            
            let weight = 1.0 / (offset + 1) as f32;
            relevance += (self.hour_counts[hour_before] as f32 * weight) / self.total_count as f32;
            if offset > 0 {
                relevance += (self.hour_counts[hour_after] as f32 * weight) / self.total_count as f32;
            }
        }
        
        relevance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        let suggestions = SmartSuggestions::new();
        
        assert_eq!(suggestions.fuzzy_match("terminal", "terminal"), 1.0);
        assert!(suggestions.fuzzy_match("termnal", "terminal") > 0.7);
        assert!(suggestions.fuzzy_match("open terminal", "terminal") > 0.5);
        assert!(suggestions.fuzzy_match("term", "terminal") > 0.5);
    }

    #[test]
    fn test_levenshtein_similarity() {
        let suggestions = SmartSuggestions::new();
        
        assert_eq!(suggestions.levenshtein_similarity("", ""), 1.0);
        assert_eq!(suggestions.levenshtein_similarity("abc", "abc"), 1.0);
        assert!(suggestions.levenshtein_similarity("abc", "abd") > 0.6);
        assert!(suggestions.levenshtein_similarity("kitten", "sitting") < 0.6);
    }
}