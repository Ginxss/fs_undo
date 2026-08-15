# fs_undo

Reversible filesystem operations for Rust.

Wraps standard filesystem operations in the enum `FsOp`, which enables reversal of the operation.
The `Undo` trait is implemented for both `FsOp` and `Vec<FsOp>` to allow for easy
reversal of an operation or a list of operations in reverse order.

On `execute()`, all information needed to fully restore the state before the operation is
queried and stored in memory.
Not efficient, but useful for scripts that operate on a limited number of filesystem entries and
want to easily restore the initial filesystem state, e.g. on error.

Symlinks are only supported on Unix systems.

## Example

```rust
use fs_undo::{fs_op::FsOp, undo::Undo};

fn main() -> std::io::Result<()> {
	let mut op = FsOp::copy_file("a.txt", "b.txt");
	op.execute()?;
	op.undo()
}
```
