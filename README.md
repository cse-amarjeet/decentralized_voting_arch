# Decentralized Voting Smart Contract for Arch Network

## Overview
This repository contains a Rust-based smart contract that implements a decentralized voting system on the Arch network. The contract allows users to create polls with customizable options, securely cast a single vote per wallet, and close polls automatically after the specified voting period ends. It emphasizes transparency, security, and gas efficiency for on-chain voting.

## Features
- **Poll Creation:** Create voting polls with a custom question and selectable options.
- **Secure Voting:** Enforces one vote per wallet to prevent double-voting using on-chain checks.
- **Real-Time Tallying:** Votes are tallied as they are cast, enabling up-to-date result displays.
- **Time-Bound Voting:** Polls have clearly defined start and end times; they can be automatically or manually closed once voting is complete.
- **Access Control:** Only authorized users (such as the poll creator) can close polls and manage administrative tasks.

## Project Structure
- **Cargo.toml:** Project manifest with dependencies and crate configurations.
- **src/**
  - `lib.rs`: Main source file containing the smart contract logic for poll creation, vote casting, and poll closure.
- **tests/**
  - Contains unit tests covering core functionality and edge cases.
- **README.md:** This documentation file.
- **.gitignore:** List of files and directories to ignore (e.g., build artifacts).

## Prerequisites
- **Rust:** Ensure you have Rust installed (preferably the latest stable version).
- **Arch Network SDK/Crates:** Include the `arch-program` crate as required.
- **Borsh:** Used for serialization and deserialization.

## Getting Started

### Installation & Build
1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/voting_smart_contract.git
   cd voting_smart_contract
   ```
2. Build the project in release mode:
   ```bash
   cargo build --release
   ```

### Running Tests
Run the unit tests to ensure everything functions as expected:
```bash
cargo test
```

## Usage
1. **Creating a Poll:**
   - Use the `CreatePoll` instruction to set up a new poll with a question, options, start time, and end time.
2. **Voting:**
   - Submit the `Vote` instruction specifying the option index. The contract ensures each wallet can only vote once.
3. **Closing a Poll:**
   - Once the voting period is over, the poll creator can call the `ClosePoll` instruction to finalize the poll and update its status.

## Contributing
Contributions are welcome! Feel free to fork the repository and open pull requests. For any issues or feature requests, please open an issue in this repository.

## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

## Hackathon Submission Details
This project is submitted as part of the weekly developer bounty program on the Arch network. It is evaluated based on functionality, security, and accuracy, along with comprehensive unit testing to verify all edge cases. Even if the code isn’t fully complete, submissions are considered for prizes and a chance to win exclusive access to the Arch recruiting channel.

---

Happy coding, and good luck with your hackathon submission!
```

This README provides a comprehensive overview, instructions, and context for your project, ensuring evaluators can easily understand your implementation and process.
