use super::*;

#[derive(Boilerplate)]
pub(crate) struct AddressHtml {
  pub(crate) address: Address,
  pub(crate) outputs: Vec<OutPoint>,
  pub(crate) sat_balance: u64,
}

impl PageContent for AddressHtml {
  fn title(&self) -> String {
    format!("Address {}", self.address)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_regex_match!(
      AddressHtml {
        address: Address::from_str(
          "bc1phuq0vkls6w926zdaem6x9n02z2gg7j2xfudgwddyey7uyquarlgsh40ev8"
        )
        .unwrap()
        .require_network(Network::Bitcoin)
        .unwrap(),
        outputs: vec![outpoint(1), outpoint(2)],
        sat_balance: 99,
      },
      "<h1>Address bc1phuq0vkls6w926zdaem6x9n02z2gg7j2xfudgwddyey7uyquarlgsh40ev8</h1>
<dl>
  <dt>sat balance</dt>
  <dd>99</dd>
  <dt>outputs</dt>
  <dd>
    <ul>
      <li><a class=monospace href=/output/1{64}:1>1{64}:1</a></li>
      <li><a class=monospace href=/output/2{64}:2>2{64}:2</a></li>
    </ul>
  </dd>
</dl>.*"
        .unindent()
    );
  }
}
