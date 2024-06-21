use std::{fmt::Debug, num::NonZeroUsize, ops::Bound, sync::mpsc};

use zond::{
    zvec::{ZVec, ZVecOperation},
    Operation, OperationType, Operations, Policy, Zond, ZondHandler,
};

struct Handler<T: OperationType>(mpsc::Sender<(usize, Operation<T>)>);

impl<T: OperationType + Debug> ZondHandler<T> for Handler<T> {
    fn handle(&self, id: usize, operations: Operations<T>) {
        for operation in operations {
            self.0.send((id, operation)).unwrap();
        }
    }
}

#[test]
pub fn zvec() {
    let (sender, reciever) = mpsc::channel();

    let zond = Zond::new(
        Handler(sender),
        Policy::on_count_operations(NonZeroUsize::new(3).unwrap()),
    );

    let mut zvec: ZVec<usize> = ZVec::new(zond.clone());
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

    let mut zvec2: ZVec<usize> = ZVec::with_capacity(5, zond);
    assert_eq!(0, zvec2.len());
    assert_eq!(5, zvec2.capacity());
    zvec2.extend_from_slice(&[1, 1, 2, 3, 5, 8, 13]);
    assert_eq!(&[1, 1, 2, 3, 5, 8, 13], zvec2.as_slice());
    zvec2.clear();
    assert_eq!(0, zvec2.len());
    zvec2.extend_from_slice(&[1, 1, 2, 3, 5, 8]);
    let vec: Vec<_> = zvec2.into();
    assert_eq!(&[1, 1, 2, 3, 5, 8], vec.as_slice());

    assert_eq!(
        format!(
            "{:?}",
            &[
                (0, ZVecOperation::New),
                (0, ZVecOperation::Push { value: 1 }),
                (0, ZVecOperation::Push { value: 2 }),
                (0, ZVecOperation::Push { value: 5 }),
                (0, ZVecOperation::Push { value: 5 }),
                (0, ZVecOperation::AsSlice),
                (
                    0,
                    ZVecOperation::ExtendFromWithin {
                        src_start_bound: Bound::Included(1),
                        src_end_bound: Bound::Unbounded
                    }
                ),
                (0, ZVecOperation::AsSlice),
                (0, ZVecOperation::Dedup),
                (0, ZVecOperation::AsSlice),
                (1, ZVecOperation::WithCapacity { capacity: 5 }),
                (1, ZVecOperation::Len),
                (1, ZVecOperation::Capacity),
                (
                    1,
                    ZVecOperation::ExtendFromSlice {
                        other: vec![1, 1, 2, 3, 5, 8, 13]
                    }
                ),
                (1, ZVecOperation::AsSlice),
                (1, ZVecOperation::Clear),
                (1, ZVecOperation::Len),
                (
                    1,
                    ZVecOperation::ExtendFromSlice {
                        other: vec![1, 1, 2, 3, 5, 8]
                    }
                ),
                (1, ZVecOperation::IntoVec),
            ]
        ),
        format!(
            "{:?}",
            reciever
                .into_iter()
                .map(|v| (v.0, v.1.get_type().clone()))
                .collect::<Vec<_>>()
        )
    );
}
