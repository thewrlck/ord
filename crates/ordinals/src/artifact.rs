use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub enum Artifact {
  Cenotaph(Cenotaph),
  Runestone(Runestone),
}

impl Artifact {
  pub fn mint(&self) -> Option<RuneId> {
    match self {
      Self::Cenotaph(cenotaph) => cenotaph.mint,
      Self::Runestone(runestone) => runestone.mint,
    }
  }

  pub fn mint_amount(&self) -> Option<u128> {
    match self {
      Self::Cenotaph(cenotaph) => cenotaph.mint_amount,
      Self::Runestone(runestone) => runestone.mint_amount,
    }
  }
}
