# if or if not to async

Async must be used in 2 spots:
* Units
* Event loop

Any code that is called in these spots should be marked as async.  If no code is executing concurrently, blocking may be
done in an async block.

An example would be in startup's initialization of the loader. `Loader::initialize` does call a blocking function
(`std::fs::read`) although as no code is running concurrently, it is allowed.

It is safe to use `SendWrapper` as the async executor is designated as a local executor (running in a single thread).
Due to the init's job being primarily IO-bound, multithreading only introduces slowdowns with the overhead of locking.

Locks should only be used where required in cases where a `RefCell` might be held over an `await` point. `Loader`
contains an `ev_lock` which should be held during the event loop to ensure unit references are only held at one point.
