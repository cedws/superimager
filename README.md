# Superimager
An Expression 2/Rust tool for rendering images in Garry's Mod at an insane speed.

![Example Render](https://i.imgur.com/5vbIr1v.jpg)

Superimager is the product of countless of hours of research and experimentation. It is capable of rendering significantly faster than other scripts by offloading as much work as possible to the backend, and writing to the screen's image buffer directly through a previously undiscovered method.

If you want to use Superimager yourself, you will need to compile the Rust backend, and run it on a server which publicly exposes port 8080. You should also modify some variables in the Expression 2 script, such as the `URL`, `Limit` and `Chunk`.

Further potential improvements includes applying any HTTP compression algorithms that Expression 2 supports, streaming, and faster resizing on the backend.