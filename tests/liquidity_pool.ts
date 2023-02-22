import {Keypair, PublicKey, Connection} from '@solana/web3.js';
import * as anchor from '@project-serum/anchor';
import assert from 'assert';

const provider = anchor.Provider.env();
anchor.setProvider(provider);
const program = anchor.workspace.LiquidityPool;

const poolValue = 100;
const winnerPrize = 80;
const ownerPrize = 20;

describe('liquidity-pool', () => {
    const owner = provider.wallet.publicKey;
    let lottery: anchor.web3.Account;
    let participant: Keypair;
    let winner: anchor.web3.Account;

    beforeEach(async () => {
        participant = Keypair.generate();
        winner = new anchor.web3.Account();
        lottery = await anchor.web3.Account.fromSeed(
            Buffer.from('liquidity-pool'),
            program.programId,
        );
    });

    describe('new_lottery', () => {
        it('should initialize the lottery with given values', async () => {
            await program.rpc.newLottery(poolValue, winnerPrize, ownerPrize, {
                accounts: {
                    owner,
                    lottery: lottery.publicKey,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    systemProgram: anchor.web3.SystemProgram.programId,
                },
                signers: [lottery],
                instructions: [await program.account.lottery.createInstruction(lottery)],
            });

            const expectedLottery: any = {
                owner,
                totalAmount: 0,
                poolValue,
                winnerPrize,
                ownerPrize,
                participants: [],
                winner: null,
            };

            const actualLottery = await program.account.lottery.fetch(lottery.publicKey);
            assert.deepStrictEqual(actualLottery, expectedLottery);
        });
    });

    describe('join_lottery', () => {
        beforeEach(async () => {
            await program.rpc.newLottery(poolValue, winnerPrize, ownerPrize, {
                accounts: {
                    owner,
                    lottery: lottery.publicKey,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    systemProgram: anchor.web3.SystemProgram.programId,
                },
                signers: [lottery],
                instructions: [await program.account.lottery.createInstruction(lottery)],
            });
        });

        it('should add participant to the lottery', async () => {
            await program.rpc.joinLottery(poolValue, {
                accounts: {
                    participant: participant.publicKey,
                    lottery: lottery.publicKey,
                },
                signers: [participant],
            });

            const expectedLottery: any = {
                owner,
                totalAmount: poolValue,
                poolValue,
                winnerPrize,
                ownerPrize,
                participants: [participant.publicKey],
                winner: null,
            };

            const actualLottery = await program.account.lottery.fetch(lottery.publicKey);
            assert.deepStrictEqual(actualLottery, expectedLottery);
        });

    });

    describe('liquidity_pool', () => {
        const connection = new Connection('http://localhost:8899', 'recent');
        const provider = new Provider(connection);
        setProvider(provider);

        // Create a new account for testing
        const owner = Keypair.generate();
        const winner = Keypair.generate();

        // Declare variables used in tests
        let pool: Pool;
        let recentSlothashes: PublicKey;

        it('initializes a new pool', async () => {
            // Initialize a new pool account
            pool = await Pool.create({
                owner: provider.wallet.publicKey,
                totalAmount: 1000,
                poolValue: 1000,
                winnerPrize: 50,
                ownerPrize: 10,
                participants: [provider.wallet.publicKey],
                winner: null,
            });

            expect(pool.owner.toBase58()).to.equal(provider.wallet.publicKey.toBase58());
            expect(pool.totalAmount).to.equal(1000);
            expect(pool.poolValue).to.equal(1000);
            expect(pool.winnerPrize).to.equal(50);
            expect(pool.ownerPrize).to.equal(10);
            expect(pool.participants).to.deep.equal([provider.wallet.publicKey]);
            expect(pool.winner).to.equal(null);
        });

        it('fails to pick a winner when no participants are in the lottery', async () => {
            const pickWinner = new Program(programId);

            try {
                await pickWinner.rpc.pickWinner({
                    accounts: {
                        lottery: pool.publicKey,
                        owner: provider.wallet.publicKey,
                        winner: winner.publicKey,
                        recentSlothashes,
                        systemProgram: System.programId,
                    },
                });
                expect.fail('Expected an error but did not receive one');
            } catch (err) {
                expect(err).to.be.an.instanceOf(Error);
                expect(err.message).to.equal('InvalidArgument');
            }
        });

        it('picks a winner and distributes the prizes', async () => {
            // Add participants to the pool
            await pool.rpc.addParticipant(provider.wallet.publicKey, {
                accounts: {
                    participant: provider.wallet.publicKey,
                },
            });

            // Set recent slothashes account
            recentSlothashes = await PublicKey.createWithSeed(
                pool.publicKey,
                'sysvar.slothashes',
                programId,
            );

            const pickWinner = new Program(programId);

            await pickWinner.rpc.pickWinner({
                accounts: {
                    lottery: pool.publicKey,
                    owner: provider.wallet.publicKey,
                    winner: winner.publicKey,
                    recentSlothashes,
                    systemProgram: System.programId,
                },
            });

            // Check that the winner and owner prizes were distributed correctly
            const winnerBalance = await provider.connection.getBalance(winner.publicKey);
            const ownerBalance = await provider.connection.getBalance(provider.wallet.publicKey);
            expect(winnerBalance).to.equal(500); // 50% of 1000
            expect(ownerBalance).to.equal(100); // 10% of 1000
            expect(pool.winner.toBase58()).to.equal(winner.publicKey.toBase58());
            expect(pool.participants).to.deep.equal([]);
        });
    });
})