use super::*;
// use near_sdk::test_utils::VMContextBuilder;
// use near_sdk::testing_env;

#[test]
fn init_contract() {
    let msg_sender = env::predecessor_account_id();
    let proposal_names = vec!["Eduction".to_string(), "Industry".to_string(), "Finance".to_string()];
    let contract = Contract::new(proposal_names.clone());

    assert_eq!(contract.chair_person, msg_sender);
    assert_eq!(
        contract.voters.get(&msg_sender).unwrap(),
        Voter {
            weight: 1,
            voted: false,
            delegate: None,
            vote: None,
        }
    );
    for (index, proposal) in contract.proposals.iter().enumerate() {
        assert_eq!(
            proposal,
            Proposal {
                name: proposal_names[index].to_string(),
                vote_count: 0,
            }
        )
    }
}

#[test]
fn give_right_to_vote() {
    // let msg_sender = env::predecessor_account_id();
    let voter_id = AccountId::new_unchecked("alice.near".to_string());
    let proposal_names = vec!["Eduction".to_string(), "Industry".to_string(), "Finance".to_string()];
    let mut contract = Contract::new(proposal_names);

    contract.give_right_to_vote(&voter_id);
    assert_eq!(
        contract.voters.get(&voter_id).unwrap(),
        Voter {
            weight: 1,
            voted: false,
            delegate: None,
            vote: None,
        }
    )
}

#[test]
fn delegate() {
    let msg_sender = env::predecessor_account_id();
    let delegate_id = Box::new(AccountId::new_unchecked("alice.near".to_string()));
    let proposal_names = vec!["Eduction".to_string(), "Industry".to_string(), "Finance".to_string()];
    let mut contract = Contract::new(proposal_names);

    contract.give_right_to_vote(&delegate_id);
    contract.delegate(*delegate_id.clone());

    assert_eq!(
        contract.voters.get(&msg_sender).unwrap().delegate,
        Some(*delegate_id)
    );
}

#[test]
fn vote() {
    let msg_sender = env::predecessor_account_id();
    let proposal_names = vec!["Eduction".to_string(), "Industry".to_string(), "Finance".to_string()];
    let mut contract = Contract::new(proposal_names);

    let vote_index = 2;
    let pre_vote_count = contract.proposals.get(vote_index).unwrap().vote_count;
    contract.vote(vote_index);

    assert_eq!(contract.voters.get(&msg_sender).unwrap().voted, true);
    assert_eq!(
        contract.voters.get(&msg_sender).unwrap().vote,
        Some(vote_index)
    );
    assert_eq!(
        contract.proposals.get(vote_index).unwrap().vote_count - pre_vote_count,
        contract.voters.get(&msg_sender).unwrap().weight
    )
}

#[test]
fn winner_name() {
    let proposal_names = vec!["Eduction".to_string(), "Industry".to_string(), "Finance".to_string()];
    let mut contract = Contract::new(proposal_names);

    let vote_index = 2;
    contract.vote(vote_index);

    assert_eq!(contract.winner_name(), Some("Finance".to_string()))
}
