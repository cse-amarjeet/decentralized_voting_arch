// src/lib.rs

use borsh::{BorshDeserialize, BorshSerialize};
use arch_program::{
    account_info::{AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
};

/// The poll state stored in an account.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Poll {
    /// Creator of the poll.
    pub creator: Pubkey,
    /// Poll question.
    pub question: String,
    /// Poll options.
    pub options: Vec<String>,
    /// Vote count per option (parallel to `options`).
    pub vote_counts: Vec<u64>,
    /// Poll start time (unix timestamp in seconds).
    pub start_time: u64,
    /// Poll end time (unix timestamp in seconds).
    pub end_time: u64,
    /// Whether the poll is closed.
    pub is_closed: bool,
    /// List of voters (to prevent double-voting).
    pub voters: Vec<Pubkey>,
}

/// Instructions the voting program accepts.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum VotingInstruction {
    /// Create a new poll.
    /// Accounts:
    ///   0. [writable] Poll account to be created.
    ///   1. [signer] Poll creator account.
    CreatePoll {
        question: String,
        options: Vec<String>,
        start_time: u64,
        end_time: u64,
    },
    /// Vote on a poll option.
    /// Accounts:
    ///   0. [writable] Poll account.
    ///   1. [signer] Voter account.
    Vote {
        option_index: u32,
    },
    /// Close a poll.
    /// Accounts:
    ///   0. [writable] Poll account.
    ///   1. [signer] Caller account (must be poll creator).
    ClosePoll,
}

entrypoint!(process_instruction);
///
/// Entry point of the program.
///
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = VotingInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        VotingInstruction::CreatePoll { question, options, start_time, end_time } => {
            process_create_poll(program_id, accounts, question, options, start_time, end_time)
        },
        VotingInstruction::Vote { option_index } => {
            process_vote(program_id, accounts, option_index)
        },
        VotingInstruction::ClosePoll => process_close_poll(program_id, accounts),
    }
}

/// Creates a new poll. Initializes the poll account with the provided data.
fn process_create_poll(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    question: String,
    options: Vec<String>,
    start_time: u64,
    end_time: u64,
) -> ProgramResult {
    // accounts[0]: poll account (writable), accounts[1]: creator (must be signer)
    let poll_account = &accounts[0];
    let creator_account = &accounts[1];

    if !creator_account.is_signer {
        msg!("Creator signature missing.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Initialize vote counts for each option.
    let vote_counts = vec![0; options.len()];
    let poll = Poll {
        creator: *creator_account.key,
        question,
        options,
        vote_counts,
        start_time,
        end_time,
        is_closed: false,
        voters: Vec::new(),
    };

    poll.serialize(&mut &mut poll_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::AccountDataTooSmall)?;

    msg!("Poll created successfully.");
    Ok(())
}

/// Casts a vote on a given poll.
fn process_vote(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    option_index: u32,
) -> ProgramResult {
    // accounts[0]: poll account (writable), accounts[1]: voter (must be signer)
    let poll_account = &accounts[0];
    let voter_account = &accounts[1];

    if !voter_account.is_signer {
        msg!("Voter signature missing.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load and deserialize the poll.
    let mut poll = Poll::try_from_slice(&poll_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if poll.is_closed {
        msg!("Poll is closed.");
        return Err(ProgramError::InvalidArgument);
    }

    // Retrieve the current time.
    let current_time = get_current_time();
    if current_time < poll.start_time || current_time > poll.end_time {
        msg!("Voting period is not active.");
        return Err(ProgramError::InvalidArgument);
    }

    // Check if the voter has already cast a vote.
    if poll.voters.contains(voter_account.key) {
        msg!("Voter has already voted.");
        return Err(ProgramError::Custom(0)); // Custom error for double voting.
    }

    // Validate the option index.
    let idx = option_index as usize;
    if idx >= poll.options.len() {
        msg!("Invalid option index.");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Increment the vote count for the selected option.
    poll.vote_counts[idx] = poll.vote_counts[idx]
        .checked_add(1)
        .ok_or(ProgramError::Custom(1))?; // Custom error for overflow.

    // Record this voter's participation.
    poll.voters.push(*voter_account.key);

    // Write the updated poll state back to the account.
    poll.serialize(&mut &mut poll_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::AccountDataTooSmall)?;

    msg!("Vote cast successfully.");
    Ok(())
}

/// Closes the poll (only allowed by the poll creator when the voting period has ended).
fn process_close_poll(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    // accounts[0]: poll account (writable), accounts[1]: caller (must be poll creator/signature)
    let poll_account = &accounts[0];
    let caller_account = &accounts[1];

    if !caller_account.is_signer {
        msg!("Caller signature missing.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load the poll.
    let mut poll = Poll::try_from_slice(&poll_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Only the poll creator can close the poll.
    if poll.creator != *caller_account.key {
        msg!("Caller is not authorized to close the poll.");
        return Err(ProgramError::IllegalOwner);
    }

    if poll.is_closed {
        msg!("Poll is already closed.");
        return Err(ProgramError::InvalidArgument);
    }

    // Retrieve current time and ensure voting period is over.
    let current_time = get_current_time();
    if current_time < poll.end_time {
        msg!("Poll voting period is still active.");
        return Err(ProgramError::InvalidArgument);
    }

    poll.is_closed = true;

    // Update the account with the closed poll.
    poll.serialize(&mut &mut poll_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::AccountDataTooSmall)?;

    msg!("Poll closed successfully.");
    Ok(())
}

/// Helper function to retrieve the current unix time (seconds).
/// In production, this should fetch the blockchain’s clock (e.g., via a sysvar).
fn get_current_time() -> u64 {
    // For demonstration purposes, return a fixed timestamp.
    // In a real deployment use the appropriate clock sysvar.
    1_620_000_000
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshSerialize;
    use std::cell::RefCell;
    use arch_program::{
        account_info::AccountInfo,
        clock::Clock,
    };

    /// A simple mock for an account.
    struct MockAccount {
        key: Pubkey,
        is_signer: bool,
        data: RefCell<Vec<u8>>,
    }

    impl MockAccount {
        fn new(key: Pubkey, size: usize, is_signer: bool) -> Self {
            Self {
                key,
                is_signer,
                data: RefCell::new(vec![0u8; size]),
            }
        }
    }

    /// Helper function to generate a dummy Pubkey.
    fn dummy_pubkey(seed: u8) -> Pubkey {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        Pubkey::new_from_array(bytes)
    }

    /// Minimal wrapper to simulate AccountInfo.
    fn create_account_info<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        data: &'a mut [u8],
    ) -> AccountInfo<'a> {
        // In a real environment the other fields would be provided by the runtime.
        AccountInfo::new(key, is_signer, true, data, &dummy_pubkey(0), false, 0)
    }

    #[test]
    fn test_create_poll() {
        let creator_key = dummy_pubkey(1);
        let poll_key = dummy_pubkey(2);
        let mut poll_data = vec![0u8; 1024]; // pre-allocated space
        let mut creator_data = vec![];

        let mut poll_account = create_account_info(&poll_key, false, &mut poll_data);
        let mut creator_account = create_account_info(&creator_key, true, &mut creator_data);

        let accounts = &mut [poll_account, creator_account];
        let question = "Best programming language?".to_string();
        let options = vec!["Rust".to_string(), "Go".to_string(), "JavaScript".to_string()];
        let start_time = 1_619_999_000;
        let end_time = 1_620_001_000;

        let instruction = VotingInstruction::CreatePoll {
            question: question.clone(),
            options: options.clone(),
            start_time,
            end_time,
        };
        let instruction_data = instruction.try_to_vec().unwrap();

        let result = process_instruction(&dummy_pubkey(0), accounts, &instruction_data);
        assert!(result.is_ok());

        let poll = Poll::try_from_slice(&accounts[0].data.borrow()).unwrap();
        assert_eq!(poll.creator, creator_key);
        assert_eq!(poll.question, question);
        assert_eq!(poll.options, options);
        assert_eq!(poll.vote_counts, vec![0, 0, 0]);
        assert_eq!(poll.start_time, start_time);
        assert_eq!(poll.end_time, end_time);
        assert_eq!(poll.is_closed, false);
        assert!(poll.voters.is_empty());
    }

    #[test]
    fn test_cast_vote() {
        let creator_key = dummy_pubkey(1);
        let voter_key = dummy_pubkey(3);
        let poll_key = dummy_pubkey(2);

        let mut poll_state = Poll {
            creator: creator_key,
            question: "Best programming language?".to_string(),
            options: vec!["Rust".to_string(), "Go".to_string(), "JavaScript".to_string()],
            vote_counts: vec![0, 0, 0],
            start_time: 1_619_999_000,
            end_time: 1_620_001_000,
            is_closed: false,
            voters: vec![],
        };

        let mut poll_data = vec![0u8; 1024];
        poll_state.serialize(&mut &mut poll_data[..]).unwrap();

        let mut voter_data = vec![];

        let mut poll_account = create_account_info(&poll_key, false, &mut poll_data);
        let mut voter_account = create_account_info(&voter_key, true, &mut voter_data);

        let accounts = &mut [poll_account, voter_account];

        // Cast a vote for the first option (index 0)
        let instruction = VotingInstruction::Vote { option_index: 0 };
        let instruction_data = instruction.try_to_vec().unwrap();
        let result = process_instruction(&dummy_pubkey(0), accounts, &instruction_data);
        assert!(result.is_ok());

        let poll_after = Poll::try_from_slice(&accounts[0].data.borrow()).unwrap();
        assert_eq!(poll_after.vote_counts[0], 1);
        assert_eq!(poll_after.voters.len(), 1);
        assert_eq!(poll_after.voters[0], voter_key);

        // Attempt to vote a second time from the same account (should fail)
        let dup_result = process_instruction(&dummy_pubkey(0), accounts, &instruction_data);
        assert!(dup_result.is_err());
    }

    #[test]
    fn test_close_poll() {
        let creator_key = dummy_pubkey(1);
        let poll_key = dummy_pubkey(2);

        let mut poll_state = Poll {
            creator: creator_key,
            question: "Best programming language?".to_string(),
            options: vec!["Rust".to_string(), "Go".to_string(), "JavaScript".to_string()],
            vote_counts: vec![3, 2, 1],
            start_time: 1_619_900_000,
            // Set end time in the past relative to our simulated current time.
            end_time: 1_619_999_000,
            is_closed: false,
            voters: vec![dummy_pubkey(3)],
        };

        let mut poll_data = vec![0u8; 1024];
        poll_state.serialize(&mut &mut poll_data[..]).unwrap();

        let mut creator_data = vec![];
        let mut poll_account = create_account_info(&poll_key, false, &mut poll_data);
        let mut creator_account = create_account_info(&creator_key, true, &mut creator_data);

        let accounts = &mut [poll_account, creator_account];
        let instruction = VotingInstruction::ClosePoll;
        let instruction_data = instruction.try_to_vec().unwrap();

        let result = process_instruction(&dummy_pubkey(0), accounts, &instruction_data);
        assert!(result.is_ok());

        let poll_after = Poll::try_from_slice(&accounts[0].data.borrow()).unwrap();
        assert!(poll_after.is_closed);
    }
}
