import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ApiKeyManager } from "../target/types/api_key_manager";
import { assert } from "chai";

describe("api_key_manager", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ApiKeyManager as Program<ApiKeyManager>;
  const issuer = provider.wallet as anchor.Wallet;
  const holder = anchor.web3.Keypair.generate();
  const serviceName = "demo-api";

  let keyAccountPda: anchor.web3.PublicKey;

  before(async () => {
    await provider.connection.requestAirdrop(holder.publicKey, 1e9);

    [keyAccountPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("api_key"),
        issuer.publicKey.toBuffer(),
        holder.publicKey.toBuffer(),
        Buffer.from(serviceName),
      ],
      program.programId
    );
  });

  it("Issues a new API key", async () => {
    const tx = await program.methods
      .issueKey(serviceName, ["read", "write"], new anchor.BN(500000))
      .accounts({
        keyAccount: keyAccountPda,
        issuer: issuer.publicKey,
        holder: holder.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("issue_key tx:", tx);
    const account = await program.account.apiKeyAccount.fetch(keyAccountPda);
    assert.equal(account.serviceName, serviceName);
    assert.deepEqual(account.permissions, ["read", "write"]);
    assert.equal(account.isRevoked, false);
    assert.equal(account.callCount.toNumber(), 0);
  });

  it("Validates the API key", async () => {
    const tx = await program.methods
      .validateKey("read")
      .accounts({ keyAccount: keyAccountPda, caller: issuer.publicKey })
      .rpc();
    console.log("validate_key tx:", tx);
    const account = await program.account.apiKeyAccount.fetch(keyAccountPda);
    assert.equal(account.callCount.toNumber(), 1);
  });

  it("Rejects wrong permission", async () => {
    try {
      await program.methods
        .validateKey("admin")
        .accounts({ keyAccount: keyAccountPda, caller: issuer.publicKey })
        .rpc();
      assert.fail("Should have thrown PermissionDenied");
    } catch (err: any) {
      assert.include(err.message, "PermissionDenied");
    }
  });

  it("Updates permissions", async () => {
    await program.methods
      .updatePermissions(["read", "write", "admin"])
      .accounts({ keyAccount: keyAccountPda, issuer: issuer.publicKey })
      .rpc();
    const account = await program.account.apiKeyAccount.fetch(keyAccountPda);
    assert.deepEqual(account.permissions, ["read", "write", "admin"]);
  });

  it("Revokes the API key", async () => {
    const tx = await program.methods
      .revokeKey()
      .accounts({ keyAccount: keyAccountPda, issuer: issuer.publicKey })
      .rpc();
    console.log("revoke_key tx:", tx);
    const account = await program.account.apiKeyAccount.fetch(keyAccountPda);
    assert.equal(account.isRevoked, true);
  });

  it("Rejects validation of revoked key", async () => {
    try {
      await program.methods
        .validateKey(null)
        .accounts({ keyAccount: keyAccountPda, caller: issuer.publicKey })
        .rpc();
      assert.fail("Should have thrown KeyRevoked");
    } catch (err: any) {
      assert.include(err.message, "KeyRevoked");
    }
  });
});