use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{Signer, SignerError},
    signers::Signers,
};

// TODO: add this to some crate in sanctum-solana-client-utils
/// newtype to impl Signers on to avoid lifetime errors from Vec::dedup()
/// Behaviour is undefined if slice passed in is not sorted in pubkey order
pub struct SortedSigners<'slice, 'signer, S: ?Sized>(pub &'slice [&'signer S]);

impl<'slice, 'signer, S: ?Sized> SortedSigners<'slice, 'signer, S> {
    pub fn iter(&self) -> SortedSignerIter<'_, '_, '_, S> {
        SortedSignerIter {
            inner: self,
            curr_i: 0,
        }
    }
}

pub struct SortedSignerIter<'a, 'slice, 'signer, S: ?Sized> {
    inner: &'a SortedSigners<'slice, 'signer, S>,
    curr_i: usize,
}

impl<'a, 'slice, 'signer, S: Signer + ?Sized> Iterator
    for SortedSignerIter<'a, 'slice, 'signer, S>
{
    type Item = &'a S;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.inner.0.get(self.curr_i)?;
        let curr_pk = curr.pubkey();
        self.curr_i += 1;
        while let Some(next) = self.inner.0.get(self.curr_i) {
            if next.pubkey() != curr_pk {
                break;
            }
            self.curr_i += 1;
        }
        Some(*curr)
    }
}

impl<'slice, 'signer, S: Signer + ?Sized> Signers for SortedSigners<'slice, 'signer, S> {
    fn pubkeys(&self) -> Vec<Pubkey> {
        self.iter().map(|s| s.pubkey()).collect()
    }

    fn try_pubkeys(&self) -> Result<Vec<Pubkey>, SignerError> {
        self.iter().map(|s| s.try_pubkey()).collect()
    }

    fn sign_message(&self, message: &[u8]) -> Vec<Signature> {
        self.iter().map(|s| s.sign_message(message)).collect()
    }

    fn try_sign_message(&self, message: &[u8]) -> Result<Vec<Signature>, SignerError> {
        self.iter().map(|s| s.try_sign_message(message)).collect()
    }

    fn is_interactive(&self) -> bool {
        self.iter().any(|s| s.is_interactive())
    }
}
