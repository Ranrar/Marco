/// Cache statistics snapshot.
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub current_size: u64,
}

impl CacheStats {
    /// Fraction of lookups that were cache hits. Returns `0.0` when no lookups have occurred.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_hit_rate_zero_on_empty() {
        let stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn smoke_hit_rate_calculation() {
        let stats = CacheStats {
            hits: 3,
            misses: 1,
            current_size: 0,
        };
        assert!((stats.hit_rate() - 0.75).abs() < f64::EPSILON);
    }
}
