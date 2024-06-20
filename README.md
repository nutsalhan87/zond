# Zond

Zond is crate with standard rust collections but with collecting statistics.

Ok, maybe it contains only analogue of `Vec` - `zvec::ZVec`. And ok, `ZVec` contains only some part of `Vec` methods. 
But I made this just for fun. I don't know anyone who would really need this.

## Overview

Let's start from constructing collection. \
Constructors similar to their std analogues' constructors but have two additional arguments:
1. `zond_collector` of type `ZondCollector`. \
  Function that consumes two arguments: `id` as `usize` and `operations` as `Operations`. All data processing is hapeppening here: 
  you can save data to file or database, send to your server or just print to console.
2. `policy` of type `policy::Policy`. \
  Desribes rules when collected operations will handled by `zond_collector`.

So at first we will implement some `ZondCollector`. It will just print operations to stdout:
```rust
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
```

Next we will construct `ZVec` that will send statistics after each three operations:
```rust
let mut zvec: ZVec<usize> = ZVec::new(
    Collector,
    Policy::OnCountOperations(OnCountOperationsMetadata::new(NonZero::new(3).unwrap())),
);
```

Finally we will execute some operations:
```rust
zvec.push(1);
zvec.push(2);
zvec.push(5);
zvec.push(5);
zvec.extend_from_within(1..);
zvec.dedup();
drop(zvec);
```

The console output will be like this:
```
0 collected
Instant { /**/ }: New
Instant { /**/ }: Push { value: 1 }
Instant { /**/ }: Push { value: 2 }

0 collected
Instant { /**/ }: Push { value: 5 }
Instant { /**/ }: Push { value: 5 }
Instant { /**/ }: ExtendFromWithin { src_start_bound: Included(1), src_end_bound: Unbounded }

0 collected
Instant { /**/ }: Dedup
Instant { /**/ }: Drop
```

As you can see, operations always being handled when dropping.