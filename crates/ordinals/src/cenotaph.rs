use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug, Default)]
pub struct Cenotaph {
  pub control: Option<RuneId>,
  pub etching: Option<Rune>,
  pub flaw: Option<Flaw>,
  pub mint: Option<RuneId>,
  pub mint_amount: Option<u128>,
}
