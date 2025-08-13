import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CapstoneFreelanceEscrow } from "../target/types/capstone_freelance_escrow";
import { assert } from "chai";

describe("capstone_freelance_escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CapstoneFreelanceEscrow as Program<CapstoneFreelanceEscrow>;

  const client = anchor.web3.Keypair.generate();
  const freelancer = anchor.web3.Keypair.generate();
  const escrowAmount = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL); // 1 SOL

  const [escrowAccountPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), client.publicKey.toBuffer(), freelancer.publicKey.toBuffer()],
    program.programId
  );

  before(async () => {
    // Airdrop SOL to the client for transactions
    const airdropTx = await provider.connection.requestAirdrop(client.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    const blockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
        signature: airdropTx,
        ...blockhash
    });
  });

  it("Initializes the escrow", async () => {
    const disputeTimeoutDays = 30;

    await program.methods
      .initializeEscrow(escrowAmount, freelancer.publicKey, disputeTimeoutDays)
      .accountsPartial({
        escrowAccount: escrowAccountPda,
        client: client.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([client])
      .rpc();

    const escrow = await program.account.escrowAccount.fetch(escrowAccountPda);
    assert.ok(escrow.client.equals(client.publicKey));
    assert.ok(escrow.freelancer.equals(freelancer.publicKey));
    assert.ok(escrow.amount.eq(escrowAmount));
    assert.deepEqual(escrow.status, { pending: {} });
  });

  it("Deposits funds into the escrow", async () => {
    await program.methods
      .depositFunds()
      .accountsPartial({
        escrowAccount: escrowAccountPda,
        client: client.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([client])
      .rpc();

    const escrow = await program.account.escrowAccount.fetch(escrowAccountPda);
    assert.deepEqual(escrow.status, { funded: {} });

    const balance = await provider.connection.getBalance(escrowAccountPda);
    assert.isAtLeast(balance, escrowAmount.toNumber());
  });

  it("Submits work", async () => {
    const workLink = "https://github.com/Some1Uknow/capstone_freelance_escrow";
    await program.methods
      .submitWork(workLink)
      .accountsPartial({
        escrowAccount: escrowAccountPda,
        freelancer: freelancer.publicKey,
      })
      .signers([freelancer])
      .rpc();

    const escrow = await program.account.escrowAccount.fetch(escrowAccountPda);
    assert.deepEqual(escrow.status, { submitted: {} });
    assert.equal(escrow.workLink, workLink);
  });

  it("Approves submission", async () => {
    await program.methods
      .approveSubmission()
      .accountsPartial({
        escrowAccount: escrowAccountPda,
        client: client.publicKey,
      })
      .signers([client])
      .rpc();

    const escrow = await program.account.escrowAccount.fetch(escrowAccountPda);
    assert.deepEqual(escrow.status, { approved: {} });
  });

  it("Withdraws payment", async () => {
    const initialFreelancerBalance = await provider.connection.getBalance(freelancer.publicKey);

    await program.methods
      .withdrawPayment()
      .accountsPartial({
        escrowAccount: escrowAccountPda,
        freelancer: freelancer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([freelancer])
      .rpc();

    const escrow = await program.account.escrowAccount.fetch(escrowAccountPda);
    assert.deepEqual(escrow.status, { complete: {} });

    const finalFreelancerBalance = await provider.connection.getBalance(freelancer.publicKey);
    assert.isAbove(finalFreelancerBalance, initialFreelancerBalance);
  });

  // Dispute/Refund Path
  describe("Dispute and Refund", () => {
    const client2 = anchor.web3.Keypair.generate();
    const freelancer2 = anchor.web3.Keypair.generate();
    const escrowAmount2 = new anchor.BN(0.5 * anchor.web3.LAMPORTS_PER_SOL);

    const [escrowAccountPda2] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), client2.publicKey.toBuffer(), freelancer2.publicKey.toBuffer()],
        program.programId
      );

    before(async () => {
        const airdropTx = await provider.connection.requestAirdrop(client2.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
        const blockhash = await provider.connection.getLatestBlockhash();
        await provider.connection.confirmTransaction({
            signature: airdropTx,
            ...blockhash
        });
    });

    it("Initializes and funds a second escrow for dispute testing", async () => {
        await program.methods
            .initializeEscrow(escrowAmount2, freelancer2.publicKey, 1)
            .accountsPartial({
                escrowAccount: escrowAccountPda2,
                client: client2.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([client2])
            .rpc();
        
        await program.methods
            .depositFunds()
            .accountsPartial({
                escrowAccount: escrowAccountPda2,
                client: client2.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([client2])
            .rpc();

        const escrow = await program.account.escrowAccount.fetch(escrowAccountPda2);
        assert.deepEqual(escrow.status, { funded: {} });
    });

    it("Initiates a dispute", async () => {
        await program.methods
            .initiateDispute()
            .accountsPartial({
                escrowAccount: escrowAccountPda2,
                client: client2.publicKey,
            })
            .signers([client2])
            .rpc();

        const escrow = await program.account.escrowAccount.fetch(escrowAccountPda2);
        assert.deepEqual(escrow.status, { disputed: {} });
    });

    it("Refunds the client after dispute", async () => {
        const initialClientBalance = await provider.connection.getBalance(client2.publicKey);

        await program.methods
            .refundClient()
            .accountsPartial({
                escrowAccount: escrowAccountPda2,
                client: client2.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([client2])
            .rpc();

        const escrow = await program.account.escrowAccount.fetch(escrowAccountPda2);
        assert.deepEqual(escrow.status, { refunded: {} });

        const finalClientBalance = await provider.connection.getBalance(client2.publicKey);
        // A small buffer for transaction fees
        assert.isAbove(finalClientBalance, initialClientBalance - 5000);
    });
  });
});
