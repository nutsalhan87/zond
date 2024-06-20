use std::{fmt::Debug, num::NonZero};

use zond::{
    policy::{OnCountOperationsMetadata, Policy},
    zvec::ZVec,
    OperationType, Operations, ZondCollector,
};

struct Collector;

impl<T: OperationType + Debug> ZondCollector<T> for Collector {
    fn zond_collect(&self, id: usize, operations: Operations<T>) {
        println!("{id} collected");
        operations
            .iter()
            .map(|v| format!("{:?}: {:?}", v.get_instant(), v.get_operation_type()))
            .for_each(|s| println!("{s}"));
        println!();
    }
}

#[test]
pub fn t() {
    let mut zvec: ZVec<usize> = ZVec::new(
        Collector,
        Policy::OnCountOperations(OnCountOperationsMetadata::new(NonZero::new(3).unwrap())),
    );
    zvec.push(1);
    zvec.push(2);
    zvec.push(5);
    zvec.push(5);
    assert_eq!(&[1, 2, 5, 5], zvec.as_slice());
    zvec.extend_from_within(1..);
    assert_eq!(&[1, 2, 5, 5, 2, 5, 5], zvec.as_slice());
    zvec.dedup();
    assert_eq!(&[1, 2, 5, 2, 5], zvec.as_slice());
    drop(zvec);

    let mut zvec2: ZVec<usize> = ZVec::with_capacity(5, Collector, Policy::OnDropOnly);
    assert_eq!(0, zvec2.len());
    assert_eq!(5, zvec2.capacity());
    zvec2.extend_from_slice(&[1, 1, 2, 3, 5, 8, 13]);
    assert_eq!(&[1, 1, 2, 3, 5, 8, 13], zvec2.as_slice());
    zvec2.clear();
    assert_eq!(0, zvec2.len());
    zvec2.extend_from_slice(&[1, 1, 2, 3, 5, 8, 13]);
    assert_eq!(&[1, 1, 2, 3, 5, 8, 13], zvec2.leak());
}
