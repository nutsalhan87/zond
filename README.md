# Zond

Zond is crate with standard rust collections but with collecting statistics.

Ok, maybe it contains only analogue of [`Vec`] - [`zvec::ZVec`]. And ok, `ZVec` contains only some part of `Vec` methods.
But I made this just for fun. I don't know anyone who would really need this.

## Example

Let's start from constructing collection.

Constructors similar to their std analogues' constructors but have additional argument - struct [`Zond`] with two fields:
1. `zond_handler` of type [`ZondHandler`]. \
 Trait object with single method that consumes two arguments: `id` as [`usize`] and `operations` as [`Operations`].
 All operations handling is hapeppening here: you can save them to file or database, send to your server or just print to console.
2. `policy` of type [`Policy`]. \
 Desribes the rules about when collected operations will handled by `zond_handler`.

So at first let's implement some ZondHandler. It will just print operations to stdout:
```rust
struct HandlerImpl;

impl<T: OperationType + Debug> ZondHandler<T> for HandlerImpl {
    fn handle(&self, id: usize, operations: Operations<T>) {
        println!("{id} collected");
        operations
            .iter()
            .for_each(|v| println!("{:?}: {:?}", v.get_instant(), v.get_type()));
        println!();
    }
}
```

Next let's construct Zond with HandlerImpl handler and such a policy that operations will be handled after each three method calls. \
It will handle operations for ZVec:
```rust
let zond: Zond<ZVecOperation<usize>> = Zond::new(
    HandlerImpl,
    Policy::on_count_operations(NonZeroUsize::new(3).unwrap()),
);
```

Next let's construct ZVec with zond variable:
```rust
let mut zvec: ZVec<usize> = ZVec::new(zond);
```

Finally let's execute some operations:
```rust
zvec.push(1);
zvec.push(2);
zvec.push(5);
zvec.push(5);
zvec.extend_from_within(1..);
zvec.dedup();
drop(zvec);
```

The console output will look like this:
```
0 collected
Instant { /* */ }: New
Instant { /* */ }: Push { value: 1 }
Instant { /* */ }: Push { value: 2 }

0 collected
Instant { /* */ }: Push { value: 5 }
Instant { /* */ }: Push { value: 5 }
Instant { /* */ }: ExtendFromWithin { src_start_bound: Included(1), src_end_bound: Unbounded }

0 collected
Instant { /* */ }: Dedup
```

As you can see, operations always being handled when dropping.