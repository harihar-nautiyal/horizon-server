# Horizon Server

Horizon Server is a command and control (C2) server for a Rust-based remote administration tool (RAT). It's designed to listen for connections from clients, manage sessions, and issue commands.

## Status

**This project is currently under active development.** Features may be incomplete or subject to change.

## Features

*   **Client Registration:** New clients can register with the server.
*   **Admin Interface:** A separate interface for administrators to manage clients.
*   **Session Management:** Track and manage active client sessions.
*   **Command Execution:** Send commands to clients and receive results.
*   **File Uploads/Downloads:** Transfer files between the server and clients.
*   **Secure Communication:** (Assuming JWT usage from `src/models/jwt.rs`) Communication can be secured using JSON Web Tokens.

## Getting Started

### Prerequisites

*   [Rust](https.www.rust-lang.org/tools/install)
*   [Cargo](https://doc.rust-lang.org/cargo/)

### Installation

1.  Clone the repository:
    ```sh
    git clone <repository-url>
    ```
2.  Navigate to the project directory:
    ```sh
    cd horizon-server
    ```

### Running the Server

To start the server, run the following command:

```sh
cargo run
```

The server will start listening for incoming connections.

## Usage

Once the server is running, clients can connect to it. Administrators can use the admin interface to list connected clients, issue commands, and manage sessions.

## Project Structure

The project is structured as follows:

```
.
├── Cargo.toml
├── src
│   ├── main.rs         # Main application entry point
│   ├── middleware.rs   # Middleware for request processing
│   ├── admin           # Admin-related functionality
│   ├── client          # Client-facing API
│   └── models          # Data structures and models
├── tests               # Integration and unit tests
└── uploads             # Directory for file uploads
```

## Disclaimer

This tool is intended for educational and research purposes only. The author is not responsible for any misuse or damage caused by this program. Use this software responsibly and only on systems you have explicit permission to access.

## Contributing

Contributions are welcome! Please feel free to submit a pull request.

## License

This project is licensed under the MIT License.
