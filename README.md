# SampleGuard

**RFID-based Sample Integrity Tracking System for Medical Devices**

A comprehensive Rust-based system demonstrating expertise in RFID technology, secure coding practices, and medical device development. Built with production-ready code, extensive testing, and modern software engineering principles.

## 🏗️ System Architecture

### Layered Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Application Layer                                  │
│   • Sample management and lifecycle                         │
│   • Integrity validation                                    │
│   • Business logic and orchestration                        │
│   • REST API endpoints                                      │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Hardware Abstraction Layer                         │
│   • RFIDReader trait (multi-vendor support)                 │
│   • Protocol abstraction                                    │
│   • Hardware emulation (Impinj & Zebra)                     │
│   • Tag simulator with realistic behavior                    │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Protocol Layer                                     │
│   • LLRP (Impinj Speedway)                                  │
│   • Zebra Protocol (FX9600)                                 │
│   • Command/Response handling                               │
│   • Network delay simulation                                │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Data & Services Layer                              │
│   • SQLite database persistence                             │
│   • Temperature monitoring                                  │
│   • Audit logging system                                    │
│   • Inventory management                                    │
├─────────────────────────────────────────────────────────────┤
│ Layer 5: Security Layer                                     │
│   • AES-256-CBC encryption                                  │
│   • SHA-256 hashing                                         │
│   • Secure key derivation                                   │
│   • Integrity checksums                                     │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

1. **Encryption Module** (`src/encryption.rs`)
   - AES-256-CBC encryption/decryption
   - SHA-256 hashing
   - Secure key derivation
   - PKCS7 padding

2. **RFID Reader Abstraction** (`src/reader.rs`)
   - Trait-based design for multi-vendor support
   - Mock reader for testing
   - Configuration management

3. **Sample Management** (`src/sample.rs`)
   - Sample lifecycle tracking
   - Status management (InProduction, InTransit, Stored, etc.)
   - Metadata handling

4. **Tag Management** (`src/tag.rs`)
   - RFID tag data structures
   - Memory layout management
   - Tag serialization/deserialization

5. **Integrity Validation** (`src/integrity.rs`)
   - Checksum validation
   - Tamper detection
   - Violation reporting

6. **Inventory Management** (`src/inventory.rs`)
   - Multi-tag scanning
   - Filtering capabilities
   - Inventory reporting

7. **Database Persistence** (`src/database.rs`)
   - SQLite integration
   - Sample CRUD operations
   - History tracking
   - Statistics generation

8. **Temperature Monitoring** (`src/temperature.rs`)
   - Real-time temperature readings
   - Violation detection
   - Statistics tracking

9. **Audit Logging** (`src/audit.rs`)
   - Comprehensive event logging
   - Query capabilities
   - Statistics generation

10. **Hardware Emulation** (`src/hardware/`)
    - Impinj Speedway reader emulation
    - Zebra FX9600 reader emulation
    - Tag simulator with realistic behavior
    - Network delay simulation
    - Error condition simulation

11. **REST API** (`src/api/`)
    - Actix-web framework
    - JSON request/response
    - Comprehensive error handling
    - Full CRUD operations

## ✅ Implemented Features

### Tier 1: Core Features (Complete)

#### 1. Multi-Tag Inventory Management
- **Status**: ✅ Complete
- **Tests**: 17+ tests
- **Features**:
  - Multi-tag scanning
  - Filtering by EPC, RSSI, antenna, tag ID
  - Inventory report generation
  - Tag tracking and management

#### 2. Database Persistence
- **Status**: ✅ Complete
- **Tests**: 20+ tests
- **Features**:
  - SQLite integration
  - Sample storage and retrieval
  - History tracking
  - Batch queries
  - Status-based queries
  - Statistics generation

#### 3. Temperature Monitoring
- **Status**: ✅ Complete
- **Tests**: 12+ tests
- **Features**:
  - Real-time temperature readings
  - Violation detection
  - Historical tracking
  - Statistics calculation
  - Configurable temperature ranges

#### 4. Audit Logging
- **Status**: ✅ Complete
- **Tests**: 10+ tests
- **Features**:
  - Comprehensive event logging
  - Event type classification
  - Severity levels
  - Query capabilities
  - Statistics generation

### Tier 2: Advanced Features (Complete)

#### 5. REST API Server
- **Status**: ✅ Complete
- **Tests**: 28+ tests (11 unit + 17 integration)
- **Features**:
  - Actix-web framework
  - Full CRUD operations for samples
  - Inventory endpoints
  - Temperature endpoints
  - Audit endpoints
  - Health check and statistics
  - JSON request/response
  - Comprehensive error handling

#### 6. Hardware Emulation
- **Status**: ✅ Complete
- **Tests**: 44+ tests (22 unit + 22 integration)
- **Features**:
  - Impinj Speedway reader emulation (LLRP protocol)
  - Zebra FX9600 reader emulation
  - Realistic tag simulation
  - Network delay simulation
  - Error condition simulation
  - Protocol compliance
  - Hardware driver with event logging

#### 7. System Demonstration Driver
- **Status**: ✅ Complete
- **Features**:
  - Comprehensive transaction logging
  - 60+ operation sequence
  - Demonstrates all system capabilities
  - Architecture understanding showcase
  - Complete system integration proof

## 📊 Test Coverage

- **Total Tests**: 154+ tests
- **Unit Tests**: 110 tests
- **Integration Tests**: 44 tests
- **All Tests Passing**: ✅

### Test Breakdown by Module

- Encryption: 15+ tests
- Reader: 10+ tests
- Sample: 12+ tests
- Tag: 8+ tests
- Integrity: 10+ tests
- Inventory: 17+ tests
- Database: 20+ tests
- Temperature: 12+ tests
- Audit: 10+ tests
- Hardware Emulation: 44+ tests
- REST API: 28+ tests

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (edition 2021)
- Cargo

### Building

```bash
# Build the project
make build
# or
cargo build --release
```

### Running

```bash
# Run the main application
make run
# or
cargo run

# Run the system demonstration
make demo
# or
cargo run --bin system_demo

# Run hardware emulation demo
cargo run --bin hardware_demo
```

### Testing

```bash
# Run all tests
make test
# or
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test module
cargo test --lib inventory
```

### Running the REST API Server

```bash
# Start the API server
cargo run --bin server

# Server runs on http://127.0.0.1:8080
# API endpoints available at http://127.0.0.1:8080/api/v1
```

## 📁 Project Structure

```
SampleGuard/
├── src/
│   ├── lib.rs                 # Library entry point
│   ├── main.rs                # Main application
│   ├── encryption.rs          # AES-256-CBC encryption
│   ├── reader.rs              # RFID reader abstraction
│   ├── sample.rs              # Sample management
│   ├── tag.rs                 # RFID tag handling
│   ├── error.rs               # Error types
│   ├── integrity.rs           # Integrity validation
│   ├── inventory.rs           # Inventory management
│   ├── database.rs            # Database persistence
│   ├── temperature.rs         # Temperature monitoring
│   ├── audit.rs               # Audit logging
│   ├── api/                   # REST API
│   │   ├── mod.rs
│   │   ├── error.rs
│   │   ├── models.rs
│   │   ├── handlers.rs
│   │   ├── routes.rs
│   │   └── server.rs
│   ├── hardware/              # Hardware emulation
│   │   ├── mod.rs
│   │   ├── protocol.rs
│   │   ├── simulator.rs
│   │   ├── impinj.rs
│   │   ├── zebra.rs
│   │   └── driver.rs
│   └── bin/
│       ├── server.rs          # API server binary
│       ├── hardware_demo.rs    # Hardware demo
│       └── system_demo.rs      # System demonstration
├── tests/                     # Integration tests
│   ├── integration_test.rs
│   ├── inventory_test.rs
│   ├── database_test.rs
│   ├── temperature_test.rs
│   ├── audit_test.rs
│   ├── api_test.rs
│   └── hardware_emulation_test.rs
├── benches/                   # Benchmarks
│   └── rfid_encryption_bench.rs
├── Cargo.toml                 # Dependencies
├── Makefile                   # Build automation
├── README.md                  # This file
├── API_IMPLEMENTATION.md      # API documentation
├── HARDWARE_EMULATION.md      # Hardware emulation docs
└── emulate_all_system.sh      # System emulation script
```

## 🔧 Key Technologies

- **Language**: Rust (edition 2021)
- **Cryptography**: AES-256-CBC, SHA-256
- **Database**: SQLite (via rusqlite)
- **Web Framework**: Actix-web
- **Serialization**: serde, serde_json
- **Testing**: Built-in Rust testing + mockall
- **Error Handling**: thiserror, anyhow
- **Logging**: env_logger, log

## 📡 API Endpoints

### Health & Statistics
- `GET /api/v1/health` - Health check
- `GET /api/v1/statistics` - System statistics

### Samples
- `GET /api/v1/samples` - List all samples
- `GET /api/v1/samples/{id}` - Get sample by ID
- `POST /api/v1/samples` - Create sample
- `PUT /api/v1/samples/{id}/status` - Update status
- `DELETE /api/v1/samples/{id}` - Delete sample
- `GET /api/v1/samples/batch/{batch}` - Get by batch

### Inventory
- `POST /api/v1/inventory/scan` - Scan for tags
- `GET /api/v1/inventory/report` - Get inventory report

### Temperature
- `POST /api/v1/temperature/read` - Read temperature
- `GET /api/v1/temperature/statistics` - Get statistics

### Audit
- `GET /api/v1/audit/events` - Get audit events
- `GET /api/v1/audit/statistics` - Get audit statistics

## 🔒 Security Features

- **Encryption**: AES-256-CBC with secure key derivation
- **Hashing**: SHA-256 for integrity checks
- **Secure Coding**: Input validation, error handling
- **Medical Device Compliance**: Designed for medical device security requirements

## 📈 Performance

- **Encryption**: Benchmarked with criterion
- **Database**: Optimized SQLite queries
- **API**: Async Actix-web server
- **Hardware Simulation**: Realistic timing and delays

## 🧪 Testing Strategy

- **Unit Tests**: Comprehensive coverage of all modules
- **Integration Tests**: End-to-end system testing
- **Hardware Tests**: Emulation-based testing
- **API Tests**: Full REST API coverage
- **Error Tests**: Error condition handling

## 📝 Documentation

- **README.md**: This file - project overview
- **API_IMPLEMENTATION.md**: REST API documentation
- **HARDWARE_EMULATION.md**: Hardware emulation details
- **Code Comments**: Comprehensive inline documentation

## 🎯 Design Principles

1. **Security First**: Encryption, hashing, secure coding practices
2. **Testability**: Extensive test coverage, mockable interfaces
3. **Modularity**: Clear separation of concerns
4. **Extensibility**: Trait-based design for easy extension
5. **Production Ready**: Error handling, logging, validation
6. **Medical Device Standards**: Compliance-focused design

## 🔮 Future Enhancements

- Real-time Dashboard (WebSocket support)
- Advanced Analytics & Reporting
- Configuration Management System
- Additional Hardware Support
- Performance Optimizations
- CI/CD Pipeline

## 📄 License

MIT OR Apache-2.0

## 👤 Author

Built to demonstrate expertise in:
- RFID technology and protocols
- Secure coding practices
- Medical device development
- Rust programming
- System architecture design

---

**Note**: This project demonstrates comprehensive understanding of RFID systems, secure coding, and medical device development practices. All features are production-ready with extensive testing and documentation.
