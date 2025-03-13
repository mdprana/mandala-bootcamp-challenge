use crate::staking::StakingConfig;
use crate::system::SystemConfig;
use std::collections::HashMap;

pub trait GovernanceConfig: StakingConfig + SystemConfig {}

pub struct Proposal<T: GovernanceConfig> {
    description: String,
    yes_votes: u32,
    no_votes: u32,
    status: ProposalStatus,
    creator: T::AccountId,  // Store the creator of the proposal
}

#[derive(Clone, PartialEq)]
pub enum ProposalStatus {
    Active,
    Approved,
    Rejected,
}

pub struct GovernancePallet<T: GovernanceConfig> {
    pub proposals: HashMap<u32, Proposal<T>>,
    pub votes: HashMap<(T::AccountId, u32), bool>, // (voter, proposal_id) -> vote_type
    next_proposal_id: u32,
}

impl<T: GovernanceConfig> GovernancePallet<T> {
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            next_proposal_id: 0,
        }
    }

    // Create a new proposal
    pub fn create_proposal(
        &mut self,
        creator: T::AccountId,
        description: String,
    ) -> Result<u32, &'static str> {
        let current_id = self.next_proposal_id;
        
        let new_proposal = Proposal {
            description,
            yes_votes: 0,
            no_votes: 0,
            status: ProposalStatus::Active,
            creator,
        };
        
        self.proposals.insert(current_id, new_proposal);
        self.next_proposal_id += 1;
        
        Ok(current_id)
    }

    // Vote on a proposal (true = yes, false = no)
    pub fn vote(
        &mut self,
        voter: T::AccountId,
        proposal_id: u32,
        vote_type: bool,
    ) -> Result<(), &'static str> {
        let vote_key = (voter.clone(), proposal_id);
        
        match self.proposals.get_mut(&proposal_id) {
            Some(proposal) => {
                if proposal.status != ProposalStatus::Active {
                    return Err("Cannot vote on inactive proposal");
                }
                
                if self.votes.contains_key(&vote_key) {
                    return Err("Voter has already cast a vote for this proposal");
                }
                
                self.votes.insert(vote_key, vote_type);
                
                match vote_type {
                    true => proposal.yes_votes += 1,  // Yes vote
                    false => proposal.no_votes += 1,  // No vote
                }
                
                Ok(())
            },
            None => Err("No proposal found with the given ID"),
        }
    }

    // Get proposal details
    pub fn get_proposal(&self, proposal_id: u32) -> Option<&Proposal<T>> {
        self.proposals.get(&proposal_id)
    }

    // Finalize a proposal (changes status based on votes)
    pub fn finalize_proposal(&mut self, proposal_id: u32) -> Result<ProposalStatus, &'static str> {
        match self.proposals.get_mut(&proposal_id) {
            Some(proposal) => {
                if proposal.status != ProposalStatus::Active {
                    return Err("Cannot finalize an already finalized proposal");
                }
                
                let new_status = if proposal.yes_votes > proposal.no_votes {
                    ProposalStatus::Approved
                } else {
                    ProposalStatus::Rejected
                };
                
                proposal.status = new_status.clone();
                
                Ok(new_status)
            },
            None => Err("No proposal found with the given ID"),
        }
    }
    
    // Get full proposal details including description and creator
    pub fn get_proposal_details(
        &self,
        proposal_id: u32,
    ) -> Result<(String, T::AccountId), &'static str> {
        match self.proposals.get(&proposal_id) {
            Some(proposal) => {
                Ok((proposal.description.clone(), proposal.creator.clone()))
            },
            None => Err("No proposal found with the given ID"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;

    #[test]
    fn test_governance_should_work() {
        let alice = 1u64;
        let bob = 2u64;
        let charlie = 3u64;

        let mut governance = GovernancePallet::<Runtime>::new();

        // Create a proposal
        let proposal_id = governance
            .create_proposal(alice, "Increase validator rewards".to_string())
            .unwrap();

        // Cast votes
        governance.vote(alice, proposal_id, true).unwrap(); // Yes vote
        governance.vote(bob, proposal_id, true).unwrap(); // Yes vote
        governance.vote(charlie, proposal_id, false).unwrap(); // No vote

        // Check proposal status before finalization
        let proposal = governance.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.yes_votes, 2);
        assert_eq!(proposal.no_votes, 1);

        // Finalize proposal
        let status = governance.finalize_proposal(proposal_id).unwrap();
        assert!(matches!(status, ProposalStatus::Approved));

        // Check proposal is now approved
        let finalized_proposal = governance.get_proposal(proposal_id).unwrap();
        assert!(matches!(
            finalized_proposal.status,
            ProposalStatus::Approved
        ));
    }
}
