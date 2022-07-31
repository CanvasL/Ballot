/*
 * smart contract `Ballot` written in RUST
 *
 *
 */
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::{env, log, near_bindgen, AccountId};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Voter {
    weight: u64,                 // weight is accumulated by delegation
    voted: bool,                 // if true, that person already voted
    delegate: Option<AccountId>, // person delegated to
    vote: Option<u64>,           // index of the voted proposal
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Proposal {
    name: String,       // proposal name
    vote_count: u64,    // number of accumulated votes
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub chair_person: AccountId,
    pub voters: UnorderedMap<AccountId, Voter>,
    pub proposals: Vector<Proposal>,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    #[private]
    pub fn new(#[serializer(borsh)] proposal_names: Vector<String>) -> Self {
        assert!(env::state_exists(), "Already initialized");

        let chair_person = env::predecessor_account_id();

        let voter = Voter {
            weight: 1,
            voted: false,
            delegate: None,
            vote: None,
        };
        let mut voters: UnorderedMap<AccountId, Voter> = UnorderedMap::new(b"m");
        voters.insert(&chair_person, &voter);

        let mut proposals: Vector<Proposal> = Vector::new(b"m");
        for proposal_name in proposal_names.iter() {
            proposals.push(&Proposal {
                name: proposal_name,
                vote_count: 0,
            })
        }

        Self {
            chair_person,
            voters,
            proposals,
        }
    }

    // Give `voter` the right to vote on this ballot.
    // May only be called by `chairperson`.
    #[payable]
    pub fn give_right_to_vote(&mut self, voter: AccountId) {
        assert!(
            self.chair_person != env::predecessor_account_id(),
            "Only chairperson can give right to vote."
        );
        assert!(
            self.voters.get(&voter).unwrap().voted,
            "The voter already voted."
        );
        assert!(
            self.voters.get(&voter).unwrap().weight == 0,
            "The voter's weight has been setted."
        );

        self.voters.get(&voter).unwrap().weight = 1;

        assert!(
            self.voters.get(&voter).unwrap().weight == 1,
            "Error setting weight."
        )
    }

    // Delegate your vote to the voter `to`.
    #[payable]
    pub fn delegate(&mut self, _to: AccountId) {
        let mut to = _to;
        let sender_voter: Voter = self.voters.get(&env::predecessor_account_id()).unwrap();

        assert!(sender_voter.weight != 0, "You have no right to vote.");
        assert!(!sender_voter.voted, "You have already voted.");
        assert!(
            to != env::predecessor_account_id(),
            "Self-delefation is disallowed."
        );

        if let Some(account) = self.voters.get(&to).unwrap().delegate {
            to = account;
            assert!(
                to != env::predecessor_account_id(),
                "Found loop in delegation."
            );
        }

        let delegate_ = self.voters.get(&to).unwrap();

        assert!(
            delegate_.weight != 0,
            "Voters cannot delegate to accounts that cannot vote."
        );

        self.voters
            .get(&env::predecessor_account_id())
            .unwrap()
            .voted = true;
        self.voters
            .get(&env::predecessor_account_id())
            .unwrap()
            .delegate = Some(to);

        if delegate_.voted {
            self.proposals
                .get(delegate_.vote.unwrap())
                .unwrap()
                .vote_count += sender_voter.weight;
        } else {
            self.voters.get(&to).unwrap().weight += sender_voter.weight;
        }
    }

    // Give your vote (including votes delegated to you)
    #[payable]
    pub fn vote(&mut self, proposals_index: u64) {
        let sender_voter: Voter = self.voters.get(&env::predecessor_account_id()).unwrap();

        assert!(sender_voter.weight != 0, "You have no right to vote.");
        assert!(!sender_voter.voted, "You have already voted.");

        self.voters
            .get(&env::predecessor_account_id())
            .unwrap()
            .voted = true;
        self.voters
            .get(&env::predecessor_account_id())
            .unwrap()
            .vote = Some(proposals_index);

        self.proposals.get(proposals_index).unwrap().vote_count = sender_voter.weight
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
