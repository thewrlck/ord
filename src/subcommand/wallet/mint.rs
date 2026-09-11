use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Mint {
  #[clap(long, help = "Mint a controlled <AMOUNT> of rune units.")]
  amount: Option<Decimal>,
  #[clap(long, help = "Use <FEE_RATE> sats/vbyte for mint transaction.")]
  fee_rate: FeeRate,
  #[clap(long, help = "Mint <RUNE>. May contain `.` or `•`as spacers.")]
  rune: SpacedRune,
  #[clap(
    long,
    help = "Include <AMOUNT> postage with mint output. [default: 10000sat]"
  )]
  postage: Option<Amount>,
  #[clap(long, help = "Send minted runes to <DESTINATION>.")]
  destination: Option<Address<NetworkUnchecked>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
  pub rune: SpacedRune,
  pub pile: Pile,
  pub mint: Txid,
}

impl Mint {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "`ord wallet mint` requires index created with `--index-runes` flag",
    );

    let rune = self.rune.rune;

    let bitcoin_client = wallet.bitcoin_client();

    let Some((id, rune_entry, _)) = wallet.get_rune(rune)? else {
      bail!("rune {rune} has not been etched");
    };

    let postage = self.postage.unwrap_or(TARGET_POSTAGE);

    let controlled = if let Some(decimal) = self.amount {
      let control = rune_entry
        .control
        .ok_or_else(|| anyhow!("rune {rune} has no control rune"))?;
      let amount = decimal.to_integer(rune_entry.divisibility)?;
      ensure!(
        amount > 0,
        "controlled mint amount must be greater than zero"
      );
      ensure!(
        rune_entry
          .open_max_supply()
          .checked_add(rune_entry.controlled_minted)
          .and_then(|supply| supply.checked_add(amount))
          .is_some(),
        "controlled mint would overflow rune supply"
      );
      Some((amount, wallet.get_rune_control_info(control)?))
    } else {
      None
    };

    let amount = if let Some((amount, _)) = &controlled {
      *amount
    } else {
      rune_entry
        .mintable(bitcoin_client.get_block_count()? + 1)
        .map_err(|err| anyhow!("rune {rune} {err}"))?
    };

    let chain = wallet.chain();

    let destination = match self.destination {
      Some(destination) => destination.require_network(chain.network())?,
      None => wallet.get_change_address()?,
    };

    ensure!(
      destination.script_pubkey().minimal_non_dust() <= postage,
      "postage below dust limit of {}sat",
      destination.script_pubkey().minimal_non_dust().to_sat()
    );

    let (runestone, inputs, outputs) = if let Some((amount, control)) = controlled {
      let runestone = Runestone {
        edicts: vec![Edict {
          id,
          amount,
          output: 1,
        }],
        mint: Some(id),
        mint_amount: Some(amount),
        pointer: Some(2),
        ..default()
      };

      let inputs = vec![TxIn {
        previous_output: control.outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      }];

      let outputs = vec![
        TxOut {
          script_pubkey: runestone.encipher(),
          value: Amount::ZERO,
        },
        TxOut {
          script_pubkey: destination.script_pubkey(),
          value: postage,
        },
        TxOut {
          script_pubkey: wallet.get_change_address()?.script_pubkey(),
          value: control.tx_out.value,
        },
      ];

      (runestone, inputs, outputs)
    } else {
      let runestone = Runestone {
        mint: Some(id),
        ..default()
      };

      let outputs = vec![
        TxOut {
          script_pubkey: runestone.encipher(),
          value: Amount::ZERO,
        },
        TxOut {
          script_pubkey: destination.script_pubkey(),
          value: postage,
        },
      ];

      (runestone, Vec::new(), outputs)
    };

    let script_pubkey = runestone.encipher();

    ensure!(
      script_pubkey.len() <= MAX_STANDARD_OP_RETURN_SIZE,
      "runestone greater than maximum OP_RETURN size: {} > {}",
      script_pubkey.len(),
      MAX_STANDARD_OP_RETURN_SIZE,
    );

    let unfunded_transaction = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: inputs,
      output: outputs,
    };

    wallet.lock_non_cardinal_outputs()?;

    let unsigned_transaction =
      fund_raw_transaction(bitcoin_client, self.fee_rate, &unfunded_transaction, None)?;

    let signed_transaction = bitcoin_client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let signed_transaction = consensus::encode::deserialize(&signed_transaction)?;

    assert_eq!(
      Runestone::decipher(&signed_transaction),
      Some(Artifact::Runestone(runestone)),
    );

    let transaction = bitcoin_client.send_raw_transaction(&signed_transaction)?;

    Ok(Some(Box::new(Output {
      rune: self.rune,
      pile: Pile {
        amount,
        divisibility: rune_entry.divisibility,
        symbol: rune_entry.symbol,
      },
      mint: transaction,
    })))
  }
}
