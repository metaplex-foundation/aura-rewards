/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  Context,
  Pda,
  PublicKey,
  Signer,
  TransactionBuilder,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import {
  Serializer,
  mapSerializer,
  struct,
  u64,
  u8,
} from '@metaplex-foundation/umi/serializers';
import {
  ResolvedAccount,
  ResolvedAccountsWithIndices,
  getAccountMetasAndSigners,
} from '../shared';

// Accounts.
export type FillVaultInstructionAccounts = {
  /** The address of the reward pool */
  rewardPool: PublicKey | Pda;
  /** The address of the reward mint */
  rewardMint: PublicKey | Pda;
  /** The address of the reward vault */
  vault: PublicKey | Pda;
  /** The address of the wallet who is responsible for filling pool's vault with rewards */
  fillAuthority: Signer;
  /** The address of the TA from which tokens will be spent */
  sourceTokenAccount: PublicKey | Pda;
  /** The address of the Token program where rewards are minted */
  tokenProgram?: PublicKey | Pda;
};

// Data.
export type FillVaultInstructionData = {
  discriminator: number;
  amount: bigint;
  distributionEndsAt: bigint;
};

export type FillVaultInstructionDataArgs = {
  amount: number | bigint;
  distributionEndsAt: number | bigint;
};

export function getFillVaultInstructionDataSerializer(): Serializer<
  FillVaultInstructionDataArgs,
  FillVaultInstructionData
> {
  return mapSerializer<
    FillVaultInstructionDataArgs,
    any,
    FillVaultInstructionData
  >(
    struct<FillVaultInstructionData>(
      [
        ['discriminator', u8()],
        ['amount', u64()],
        ['distributionEndsAt', u64()],
      ],
      { description: 'FillVaultInstructionData' }
    ),
    (value) => ({ ...value, discriminator: 1 })
  ) as Serializer<FillVaultInstructionDataArgs, FillVaultInstructionData>;
}

// Args.
export type FillVaultInstructionArgs = FillVaultInstructionDataArgs;

// Instruction.
export function fillVault(
  context: Pick<Context, 'programs'>,
  input: FillVaultInstructionAccounts & FillVaultInstructionArgs
): TransactionBuilder {
  // Program ID.
  const programId = context.programs.getPublicKey(
    'mplxRewards',
    'BF5PatmRTQDgEKoXR7iHRbkibEEi83nVM38cUKWzQcTR'
  );

  // Accounts.
  const resolvedAccounts = {
    rewardPool: {
      index: 0,
      isWritable: true as boolean,
      value: input.rewardPool ?? null,
    },
    rewardMint: {
      index: 1,
      isWritable: false as boolean,
      value: input.rewardMint ?? null,
    },
    vault: {
      index: 2,
      isWritable: true as boolean,
      value: input.vault ?? null,
    },
    fillAuthority: {
      index: 3,
      isWritable: false as boolean,
      value: input.fillAuthority ?? null,
    },
    sourceTokenAccount: {
      index: 4,
      isWritable: false as boolean,
      value: input.sourceTokenAccount ?? null,
    },
    tokenProgram: {
      index: 5,
      isWritable: false as boolean,
      value: input.tokenProgram ?? null,
    },
  } satisfies ResolvedAccountsWithIndices;

  // Arguments.
  const resolvedArgs: FillVaultInstructionArgs = { ...input };

  // Default values.
  if (!resolvedAccounts.tokenProgram.value) {
    resolvedAccounts.tokenProgram.value = context.programs.getPublicKey(
      'splToken',
      'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'
    );
    resolvedAccounts.tokenProgram.isWritable = false;
  }

  // Accounts in order.
  const orderedAccounts: ResolvedAccount[] = Object.values(
    resolvedAccounts
  ).sort((a, b) => a.index - b.index);

  // Keys and Signers.
  const [keys, signers] = getAccountMetasAndSigners(
    orderedAccounts,
    'programId',
    programId
  );

  // Data.
  const data = getFillVaultInstructionDataSerializer().serialize(
    resolvedArgs as FillVaultInstructionDataArgs
  );

  // Bytes Created On Chain.
  const bytesCreatedOnChain = 0;

  return transactionBuilder([
    { instruction: { keys, programId, data }, signers, bytesCreatedOnChain },
  ]);
}
