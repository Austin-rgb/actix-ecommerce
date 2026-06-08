# Actix E-Commerce

A modern e-commerce platform built with **Rust** and the **Actix-web** framework, featuring a responsive HTML frontend and containerized deployment.

## 📋 Overview

This project is a full-stack e-commerce application that combines:
- **Backend**: High-performance Rust API using Actix-web
- **Frontend**: HTML/CSS responsive user interface
- **Infrastructure**: Docker containerization for easy deployment
- **Utilities**: Python scripts for various tasks

## 🛠️ Tech Stack

- **Rust** (18.8%) - High-performance backend with Actix-web framework
- **HTML** (75.1%) - Frontend markup and user interface
- **Python** (4.3%) - Utility scripts and automation
- **Docker** (1.8%) - Container orchestration and deployment

## ✨ Features

- Modern e-commerce platform architecture
- High-performance backend with Rust
- Responsive HTML frontend
- Docker support for containerized deployment
- RESTful API endpoints

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+
- Docker (optional, for containerized deployment)
- Python 3.8+ (for utility scripts)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/Austin-rgb/actix-ecommerce.git
   cd actix-ecommerce
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the application**
   ```bash
   cargo run
   ```

### Docker Deployment

Build and run with Docker:

```bash
docker build -t actix-ecommerce .
docker run -p 8080:8080 actix-ecommerce
```

## 📁 Project Structure

```
actix-ecommerce/
├── src/                 # Rust backend source code
├── static/              # HTML/CSS frontend files
├── scripts/             # Python utility scripts
├── Dockerfile           # Docker configuration
├── Cargo.toml           # Rust dependencies
└── README.md           # This file
```

## 🔧 Configuration

Configuration details and environment setup can be found in the respective source files. Ensure you have the required dependencies installed before running the application.

## 📝 Usage

The application runs an e-commerce platform with:
- Product browsing and management
- Shopping cart functionality
- Order processing
- User management

For API documentation and detailed endpoints, refer to the inline code comments or generated API docs.

## 🤝 Contributing

Contributions are welcome! To contribute:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## 📄 License

This project is open source and available for public use.

## 📞 Contact

For issues, questions, or suggestions, please open an issue on the [GitHub repository](https://github.com/Austin-rgb/actix-ecommerce).

---

**Last Updated**: June 2026  
**Repository**: [Austin-rgb/actix-ecommerce](https://github.com/Austin-rgb/actix-ecommerce)
