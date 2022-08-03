#[cfg(test)]
mod test;

use near_sdk::__private::BorshIntoStorageKey;
/*
 * smart contract `Ballot` written in RUST
 *
 *
 */
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::{env, log, near_bindgen, AccountId};

#[derive(BorshSerialize)]
enum StorageKey {
    VoterTag,
    ProposalsTag,
}
impl BorshIntoStorageKey for StorageKey {}

// #[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug)]
pub struct Voter {
    weight: u64,                 // weight is accumulated by delegation
    voted: bool,                 // if true, that person already voted
    delegate: Option<AccountId>, // person delegated to
    vote: Option<u64>,           // index of the voted proposal
}

// #[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub struct Proposal {
    name: String,    // proposal name
    vote_count: u64, // number of accumulated votes
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Contract {
    pub chair_person: AccountId,
    pub voters: UnorderedMap<AccountId, Voter>,
    pub proposals: Vector<Proposal>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            chair_person: AccountId::new_unchecked("test.near".to_string()),
            voters: UnorderedMap::new(StorageKey::VoterTag),
            proposals: Vector::new(StorageKey::ProposalsTag)
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    #[private]
    pub fn new(proposal_names: &[&'static str]) -> Self {
        assert!(!env::state_exists(), "Already initialized");

        let chair_person = env::predecessor_account_id();

        let voter = Voter {
            weight: 1,
            voted: false,
            delegate: None,
            vote: None,
        };
        let mut voters: UnorderedMap<AccountId, Voter> = UnorderedMap::new(StorageKey::VoterTag);
        voters.insert(&chair_person, &voter);

        let mut proposals: Vector<Proposal> = Vector::new(StorageKey::ProposalsTag);
        for proposal_name in proposal_names.iter() {
            proposals.push(&Proposal {
                name: proposal_name.to_string(),
                vote_count: 0,
            })
        }

        log!("Contract Ballot initialized.");

        Self {
            chair_person,
            voters,
            proposals,
        }
    }

    // Give `voter` the right to vote on this ballot.
    // May only be called by `chairperson`.
    #[payable]
    pub fn give_right_to_vote(&mut self, voter: &AccountId) {
        assert!(
            self.chair_person == env::predecessor_account_id(),
            "Only chairperson can give right to vote."
        );

        if let Some(val) = self.voters.get(voter) {
            assert!(!val.voted, "The voter already voted.");
            self.voters.get(voter).unwrap().weight = 1;
        } else {
            self.voters.insert(
                voter,
                &Voter {
                    weight: 1,
                    voted: false,
                    delegate: None,
                    vote: None,
                },
            );
        }
        log!("Chair person have give right to Voter {}", voter);
    }

    // Delegate your vote to the voter `to`.
    #[payable]
    pub fn delegate(&mut self, _to: AccountId) {
        let to = Box::new(_to);
        let sender_voter: Voter = self.voters.get(&env::predecessor_account_id()).unwrap();

        assert!(sender_voter.weight != 0, "You have no right to vote.");
        assert!(!sender_voter.voted, "You have already voted.");
        assert!(
            *to != env::predecessor_account_id(),
            "Self-delefation is disallowed."
        );

        if let Some(delegate_voter) = self.voters.get(&to) {
            assert!(
                delegate_voter.weight != 0,
                "Voters cannot delegate to accounts that cannot vote."
            );

            if let Some(account) = delegate_voter.delegate {
                assert!(
                    account != env::predecessor_account_id(),
                    "Found loop in delegation."
                );
            }

            let mut temp_voter = self.voters.get(&env::predecessor_account_id()).unwrap();
            temp_voter.voted = true;
            temp_voter.delegate = Some(*to.clone());
            self.voters
                .insert(&env::predecessor_account_id(), &temp_voter);

            if delegate_voter.voted {
                let mut temp_proposal = self.proposals.get(delegate_voter.vote.unwrap()).unwrap();
                temp_proposal.vote_count += sender_voter.weight;

                self.proposals
                    .replace(delegate_voter.vote.unwrap(), &temp_proposal);
            } else {
                let mut temp_voter = self.voters.get(&to).unwrap();
                temp_voter.weight += sender_voter.weight;

                self.voters.insert(&to, &temp_voter);
            }
        } else {
            panic!("delegate account has no right to vote.")
        }
    }

    // Give your vote (including votes delegated to you)
    #[payable]
    pub fn vote(&mut self, proposals_index: u64) {
        let sender_voter: Voter = self.voters.get(&env::predecessor_account_id()).unwrap();

        assert!(sender_voter.weight != 0, "You have no right to vote.");
        assert!(!sender_voter.voted, "You have already voted.");

        let mut temp_voter = self.voters.get(&env::predecessor_account_id()).unwrap();
        temp_voter.voted = true;
        temp_voter.vote = Some(proposals_index);
        let mut temp_proposal = self.proposals.get(proposals_index).unwrap();
        temp_proposal.vote_count += sender_voter.weight;

        self.voters.insert(&env::predecessor_account_id(), &temp_voter);
        self.proposals.replace(proposals_index, &temp_proposal);

        log!("{} vote to proposal {}", env::predecessor_account_id(), proposals_index)
    }

    // Calls winningProposal() function to get the index
    // of the winner contained in the proposals array and then
    // returns the name of the winner
    pub fn winner_name(&self) -> Option<String> {
        let result = self.winning_proposal();
        if let Some(index) = result {
            Some(self.proposals.get(index).unwrap().name)
        } else {
            None
        }
    }

    // Computes the winning proposal taking all
    #[private]
    fn winning_proposal(&self) -> Option<u64> {
        let mut winning_proposal_index: Option<u64> = None;
        let mut winning_vote_count: u64 = 0;
        for (i, proposal) in self.proposals.iter().enumerate() {
            if proposal.vote_count > winning_vote_count {
                winning_vote_count = proposal.vote_count;
                winning_proposal_index = Some(i as u64);
            }
        }
        winning_proposal_index
    }
}
