# Whati8 - Bill Splitter App

Whati8 is a bill splitter app written in Rust and Leptos, designed to run as a WebAssembly (Wasm) binary on the client-side. With Whati8, users can easily split bills and expenses among friends, making the process of managing shared expenses simple and efficient.

## Features

- **User-Friendly Interface**: Intuitive and easy-to-use interface for seamless bill splitting.
- **Real-Time Updates**: Enjoy real-time updates as you add or remove participants, ensuring everyone is on the same page.
- **Client-Side Execution**: Leverage the power of WebAssembly to run the app entirely on the client-side, providing a fast and responsive experience.

## Technologies Used

- **Rust**: The core programming language for building the application logic.
- **Leptos**: A reactive programming library for Rust, facilitating reactive and dynamic user interfaces.
- **WebAssembly (Wasm)**: Enables the app to run in the browser, providing performance benefits.

## Getting Started

To run Whati8 locally, follow these steps:

1. Clone the repository:

    ```bash
    git clone https://github.com/your-username/whati8.git
    ```

2. Build the Wasm binary:

    ```bash
    cd whati8
    cargo build --target wasm32-unknown-unknown
    ```

3. Serve the app using a static file server. You can use tools like `basic-http-server`:

    ```bash
    basic-http-server ./target/wasm32-unknown-unknown/debug/
    ```

4. Open your browser and navigate to `http://localhost:4000` to access Whati8.

## Installing Additional Tools

By default, `cargo-leptos` uses `nightly` Rust, `cargo-generate`, and `sass`. If you run into any trouble, you may need to install one or more of these tools.

1. `rustup toolchain install nightly --allow-downgrade` - make sure you have Rust nightly
2. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust to WebAssembly
3. `cargo install cargo-generate` - install `cargo-generate` binary (should be installed automatically in future)
4. `npm install -g sass` - install `dart-sass` (should be optional in future)

## Executing a Server on a Remote Machine Without the Toolchain
After running a `cargo leptos build --release` the minimum files needed are:

1. The server binary located in `target/server/release`
2. The `site` directory and all files within located in `target/site`

Copy these files to your remote server. The directory structure should be:
```text
leptos_start
site/
```
Set the following environment variables (updating for your project as needed):
```sh
export LEPTOS_OUTPUT_NAME="leptos_start"
export LEPTOS_SITE_ROOT="site"
export LEPTOS_SITE_PKG_DIR="pkg"
export LEPTOS_SITE_ADDR="127.0.0.1:3000"
export LEPTOS_RELOAD_PORT="3001"
```
Finally, run the server binary.


## Usage

1. **Input Bill Details**: Enter the bill amount and details.
2. **Add Participants**: Include participants by providing their names.
3. **Adjust Amounts**: Easily split the bill or customize amounts for each participant.
4. **Real-Time Updates**: Watch as the app dynamically updates based on your input.

## Contributions

Contributions are welcome! Feel free to open issues, submit pull requests, or provide feedback to enhance Whati8.

## License

This project is licensed under the [MIT License](LICENSE).




