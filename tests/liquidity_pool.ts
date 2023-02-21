import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { LiquidityPool } from "../target/types/liquidity_pool";

describe("liquidity_pool", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.LiquidityPool as Program<LiquidityPool>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
