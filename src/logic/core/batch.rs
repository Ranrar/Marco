use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use crate::logic::ast::blocks_and_inlines::Block;
use crate::logic::core::parser::EventIter;

/// Parse a batch of documents in parallel using a fixed-size Rayon thread pool.
pub fn parse_batch_parallel<'a>(docs: Vec<&'a Block>) -> Vec<usize> {
    // Create a thread pool with a fixed number of threads (e.g., num_cpus)
    let pool = ThreadPoolBuilder::new().num_threads(num_cpus::get()).build().unwrap();
    pool.install(|| {
        docs.par_iter()
            .map(|block| {
                let mut iter = EventIter::new(block, None);
                let mut count = 0;
                while let Some(_event) = iter.next() {
                    count += 1;
                }
                count
            })
            .collect::<Vec<_>>()
    })
}

/// NOTE: All GTK/UI code must run on the main thread!
/// Do NOT call any GTK functions from within this batch function or any Rayon worker thread.
/// Use message passing (e.g., glib::Sender) to communicate results to the main thread if needed.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::ast::blocks_and_inlines::{Block, LeafBlock};
    #[test]
    fn test_parallel_batch_streaming() {
        let docs = vec![
            Block::Leaf(LeafBlock::Paragraph(vec![], None)),
            Block::Leaf(LeafBlock::Paragraph(vec![], None)),
        ];
        let results = parse_batch_parallel(docs.iter().collect());
        assert_eq!(results.len(), 2);
        for count in results {
            assert!(count > 0);
        }
    }
}
