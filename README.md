# Windows Internals in Rust

&gt; *"Give me a place to stand, and I will move the world"* - Archimedes, probably talking about Windows Job Objects

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://rust-lang.org)
[![Windows](https://img.shields.io/badge/platform-Windows-blue)](https://microsoft.com)
[![Unsafe](https://img.shields.io/badge/unsafe-yes%2C%20but%20carefully-red)](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)

Because sometimes `std::process` just isn't close enough to the metal.

## What is this?

A Rust exploration of Windows NT internals, implementing:

- **Processes** - The containers of suffering (and virtual memory)
- **Threads** - Those things that race conditions are made of
- **Job Objects** - Like a nanny, but for processes

All wrapped in safe-ish Rust abstractions that won't (shouldn't) bluescreen your machine.

## Quick Start

```bash
# Clone and build
git clone https://github.com/Guap-Codes/Windows-Internals-in-Rust.git

cd windows-internals

cargo build --release
```

# Run examples
cargo run -- sandbox      #  Sandboxed process
cargo run -- threads      #  Thread pool demo  
cargo run -- spawn        #  Process spawner
cargo run -- inject       #  Memory operations (educational!)
cargo run -- all          #  Everything at once


###  Architecture
```
┌─────────────────────────────────────┐
│           Your Code                 │
├─────────────────────────────────────┤
│    Safe Rust API (Process, etc)     │
├─────────────────────────────────────┤
│    windows-rs crate (FFI)           │
├─────────────────────────────────────┤
│    ntdll.dll / kernel32.dll         │
├─────────────────────────────────────┤
│    Windows NT Kernel                │
└─────────────────────────────────────┘
         ↓
    [HARDWARE ABSTRACTION LAYER]
         ↓
     (hopefully not here)
```

### Project Structure
```
windows-internals/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs                 # Entry point with examples
│   ├── lib.rs                  # Library exports
│   ├── core/
│   │   ├── mod.rs              # Core module exports
│   │   ├── process.rs          # Process implementation
│   │   ├── thread.rs           # Thread implementation
│   │   └── job.rs              # Job object implementation
│   ├── utils/
│   │   ├── mod.rs              # Utilities
│   │   ├── errors.rs           # Error types
│   │   └── conversions.rs      # String/FFI conversions
│   └── examples/
│       ├── mod.rs              # Examples module
│       ├── process_spawner.rs  # Process creation demo
│       ├── thread_pool.rs      # Thread management demo
│       ├── sandbox.rs          # Job object sandbox demo
│       └── injector.rs         # DLL injection demo (educational)
```

## Key Concepts
* Processes

    Containers with private virtual address space
    Own handles, security context, and at least one thread
    Expensive to create (unlike Linux fork())

* Threads

    Scheduled by Windows kernel
    Share process memory (convenient and terrifying)
    Each has two stacks (user + kernel mode)

* Job Objects

    Group processes for bulk management
    Enforce resource limits (RAM, CPU time, process count)
    Kill-on-close: Like try {} finally { destroy_everything() }

## Examples

- Creating a Sandbox:
```
let sandbox = JobObject::create(Some("MySandbox"))?;
sandbox.set_limits(&JobLimits {
    max_working_set: Some(100 * 1024 * 1024), // 100MB max
    active_process_limit: Some(5),
    kill_on_close: true,
    ..Default::default()
})?;

let (proc, thread) = Process::create("untrusted.exe", None, true)?;
sandbox.assign(&proc)?;  // Now it's trapped
thread.resume()?;
```

- Spawning Threads

```
unsafe extern "system" fn worker(_: *mut c_void) -> u32 {
    println!("Hello from thread!");
    0
}

let thread = Thread::create(
    Some(worker as LPTHREAD_START_ROUTINE),
    ptr::null(),
    Default::default()
)?;
thread.wait(None)?;
```

## Safety & Ethics

    Educational Use Only: The injection example demonstrates concepts, don't be evil
    Administrator Privileges: Some operations require elevation
    Anti-Virus: Your AV might get excited about CreateRemoteThread examples

## Building
Requirements:

    Windows 10/11
    Rust 1.70+
    Visual Studio Build Tools (or MinGW)

```bash
cargo build --release
# Output: target/release/windows-internals.exe
```

## License
MIT - See [LICENSE|(LICENSE)]

## Acknowledgments

  -  Windows Internals books by Russinovich et al.
  -  The windows-rs team for making FFI bearable
  -  Coffee

Built with rust🦀 and questionable life choices


## Compilation Instructions

1. **Install Rust** (https://rustup.rs/)
2. **Install Visual Studio Build Tools** (Windows SDK required)
3. **Create project structure** as shown above
4. **Copy all code** into respective files
5. **Run**: `cargo run --release`

The executable will be at `target/release/windows-internals.exe` and can be run on any Windows 10/11 machine without Rust installed (though some features need admin privileges).
