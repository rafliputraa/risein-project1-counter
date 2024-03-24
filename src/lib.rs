mod instructions;

use crate::instructions::CounterInstructions;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct CounterAccount {
    pub counter: u32,
}

trait InputProvider {
    fn get_input(&self) -> String;
}

struct StdInputProvider;

impl InputProvider for StdInputProvider {
    fn get_input(&self) -> String {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        input.trim().to_string()
    }
}

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instructions_data: &[u8],
) -> ProgramResult {
    msg!("Counter program entry point");

    let instruction: CounterInstructions = CounterInstructions::unpack(instructions_data)?;
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    let mut counter_account = CounterAccount::try_from_slice(&account.data.borrow())?;

    match instruction {
        CounterInstructions::Increment(args) => {
            counter_account.counter += args.value;
        }
        CounterInstructions::Decrement(args) => {
            if counter_account.counter >= args.value {
                counter_account.counter -= args.value
            } else {
                counter_account.counter = 0
            }
        }
        CounterInstructions::Reset => {
            counter_account.counter = 0;
        }
        CounterInstructions::Update(args) => {
            counter_account.counter = args.value;
        }
    }

    counter_account.serialize(&mut &mut account.data.borrow_mut()[..])?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_program::{clock::Epoch, lamports, pubkey::Pubkey};
    use std::mem;

    struct StubIncrementInputProvider;
    struct StubDecrementInputProvider;
    struct StubDecrementInputProviderGreaterThanCurrentValue;

    impl InputProvider for StubIncrementInputProvider {
        fn get_input(&self) -> String {
            "20".to_string()
        }
    }
    impl InputProvider for StubDecrementInputProvider {
        fn get_input(&self) -> String {
            "10".to_string()
        }
    }
    impl InputProvider for StubDecrementInputProviderGreaterThanCurrentValue {
        fn get_input(&self) -> String {
            "30".to_string()
        }
    }

    #[test]
    fn test_counter() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = vec![0; mem::size_of::<u32>()];
        let owner = Pubkey::default();

        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            Epoch::default(),
        );

        let accounts = vec![account];

        let mut increment_instruction_data = vec![0];
        let mut decrement_instruction_data = vec![1];
        let mut decrement_instruction_data_gt_current_value = vec![1];
        let mut update_instruction_data = vec![2];
        let reset_instruction_data = vec![3];

        let parsed_increment_value: u32 = match StubIncrementInputProvider.get_input().parse() {
            Ok(value) => value,
            Err(_) => {
                println!("Failed to parse the string into a u32");
                return; // or handle the error in some appropriate way
            }
        };
        increment_instruction_data.extend_from_slice(&parsed_increment_value.to_le_bytes());
        process_instruction(&program_id, &accounts, &increment_instruction_data).unwrap();
        assert_eq!(
            CounterAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            20
        );

        let parsed_decrement_value: u32 = match StubDecrementInputProvider.get_input().parse() {
            Ok(value) => value,
            Err(_) => {
                println!("Failed to parse the string into a u32");
                return; // or handle the error in some appropriate way
            }
        };
        decrement_instruction_data.extend_from_slice(&parsed_decrement_value.to_le_bytes());
        process_instruction(&program_id, &accounts, &decrement_instruction_data).unwrap();
        assert_eq!(
            CounterAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            10
        );

        let parsed_decrement_value_gt_current_value: u32 = match StubDecrementInputProviderGreaterThanCurrentValue.get_input().parse() {
            Ok(value) => value,
            Err(_) => {
                println!("Failed to parse the string into a u32");
                return; // or handle the error in some appropriate way
            }
        };
        decrement_instruction_data_gt_current_value.extend_from_slice(&parsed_decrement_value_gt_current_value.to_le_bytes());
        process_instruction(&program_id, &accounts, &decrement_instruction_data_gt_current_value).unwrap();
        assert_eq!(
            CounterAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            0
        );

        let update_value = 33u32;
        update_instruction_data.extend_from_slice(&update_value.to_le_bytes());

        process_instruction(&program_id, &accounts, &update_instruction_data).unwrap();
        assert_eq!(
            CounterAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            33
        );

        process_instruction(&program_id, &accounts, &reset_instruction_data).unwrap();
        assert_eq!(
            CounterAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            0
        );
    }
}
