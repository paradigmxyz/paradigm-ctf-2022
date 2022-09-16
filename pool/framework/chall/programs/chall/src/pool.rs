use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Pool {
    pub seed: u64,
    pub token_mint: Pubkey,
    pub redeem_tokens_mint_bump: u8,
    pub token_account_bump: u8,
    pub withdrawal_queue_header_bump: u8,
}

#[account]
#[derive(Default)]
pub struct WithdrawalQueueHeader {
    head_node: Option<Pubkey>,
    tail_node: Option<Pubkey>,
    // used as part of the seed of new queue nodes
    pub nonce: u64,
}

impl WithdrawalQueueHeader {
    /// Pushes a node to the end of the queue
    pub fn push(&mut self, node: &mut Account<WithdrawalQueueNode>, prev_node: Option<&AccountInfo>) -> Result<()> {
        if self.empty() {
            assert!(prev_node.is_none());
            self.head_node = Some(node.key());
            node.prev_node = None;
        } else {
            assert!(prev_node.is_some());
            let mut prev_node_account: Account<WithdrawalQueueNode> = Account::try_from(prev_node.unwrap())?;
            assert_eq!(self.tail_node, Some(prev_node_account.key()));
            prev_node_account.next_node = Some(node.key());
            node.prev_node = Some(prev_node_account.key());
            save_account::<WithdrawalQueueNode>(&prev_node_account.to_account_info(), &mut prev_node_account)?;
        }

        self.tail_node = Some(node.key());
        self.incr_nonce();
        Ok(())
    }

    /// Pops a node from the front of the queue
    pub fn pop(&mut self, node: &mut Account<WithdrawalQueueNode>) -> Result<()> {
        assert_eq!(self.head_node, Some(node.key()));
        self.head_node = node.next_node;
        Ok(())
    }

    pub fn empty(&self) -> bool {
        self.head_node.is_none()
    }

    /// Removes a node from the queue
    pub fn remove(&mut self, node: &Account<WithdrawalQueueNode>, prev_node: Option<&AccountInfo>) -> Result<()> {
        assert!(!self.empty());

        match self.head_node {
            Some(head_node_key) => {
                if head_node_key == node.key() {
                    self.head_node = node.next_node;
                }
            }
            None => panic!()
        }

        match self.tail_node {
            Some(tail_node_key) => {
                if tail_node_key == node.key() {
                    self.tail_node = node.prev_node;
                }
            }
            None => panic!()
        }

        if node.prev_node.is_some() {
            let prev_node_account_info = prev_node.unwrap();
            let mut prev_node_account: Account<WithdrawalQueueNode> = Account::try_from(prev_node_account_info)?;
            prev_node_account.next_node = node.next_node;
            save_account(&prev_node_account.to_account_info(), &mut prev_node_account)?;
        }

        self.incr_nonce();
        Ok(())
    }

    fn incr_nonce(&mut self) {
        self.nonce = self.nonce.checked_add(1).unwrap();
    }
}

#[account]
#[derive(Default)]
pub struct WithdrawalQueueNode {
    pub user: Pubkey,
    pub amount: u64,
    prev_node: Option<Pubkey>,
    next_node: Option<Pubkey>,
    pub nonce: u64,
}

pub fn save_account<T: AccountSerialize + AccountDeserialize + Clone + Owner>(
    account_info: &AccountInfo,
    account: &mut Account<'_, T>,
) -> Result<()> {
    let mut data = account_info.try_borrow_mut_data()?;
    let dst: &mut [u8] = &mut data;
    let mut cursor = std::io::Cursor::new(dst);
    account.try_serialize(&mut cursor)?;
    Ok(())
}
