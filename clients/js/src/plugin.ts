import { UmiPlugin } from '@metaplex-foundation/umi';
import { createRewardsProgram } from './generated';

export const Rewards = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createRewardsProgram(), false);
  },
});
