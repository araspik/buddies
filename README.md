# Buddies: A low-level buddy allocator

Buddies provides a low-level and unsafe buddy allocator to work with - however,
making it safe is quite simple. All that needs to be done is to store extra
information that does the following things:
1. Ensures that all allocations have shorter lifetimes than the allocator
2. Ensures that multiple mutable allocations are possible simultaneously
3. Ensures that allocations are dropped correctly.
The primitives are provided to do this - see `Buddies::allocate` and
`Buddies::free`.

It does not require `std`, and will remain like this (so that bare-metal
kernels and applications can use it easily).
