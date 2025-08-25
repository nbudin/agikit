use std::collections::{HashMap, HashSet, VecDeque};

use crate::logic::{LogicCondition, LogicGoto, LogicInstruction, asm::codegen::generate_labels};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostCompilationPassResult {
    Unchanged = 0,
    Changed = 1,
}

pub trait PostCompilationPass {
    fn run_once(&mut self, instructions: &mut Vec<LogicInstruction>) -> PostCompilationPassResult;

    fn run_until_done(&mut self, instructions: &mut Vec<LogicInstruction>) {
        let mut result = PostCompilationPassResult::Changed;
        while result == PostCompilationPassResult::Changed {
            result = self.run_once(instructions);
        }
    }
}

impl<F: FnMut(&mut Vec<LogicInstruction>) -> PostCompilationPassResult> PostCompilationPass for F {
    fn run_once(&mut self, instructions: &mut Vec<LogicInstruction>) -> PostCompilationPassResult {
        (self)(instructions)
    }
}

pub fn remove_unreachable_instructions(
    instructions: &mut Vec<LogicInstruction>,
) -> PostCompilationPassResult {
    let mut result = PostCompilationPassResult::Unchanged;
    let labels = generate_labels(instructions, &[]);
    let label_addresses = labels.iter().map(|l| l.address).collect::<HashSet<_>>();
    let mut reachable = true;

    instructions.retain(|instruction| {
        if label_addresses.contains(&instruction.address()) {
            reachable = true;
        }

        let instruction_was_reachable = reachable;
        if let LogicInstruction::Goto(_) = instruction {
            reachable = false;
        }

        if !instruction_was_reachable {
            result = PostCompilationPassResult::Unchanged;
        }

        instruction_was_reachable
    });

    result
}

pub struct RemoveRedundantGotoInstructionsPass {
    address_remappings: HashMap<u16, u16>,
}

impl RemoveRedundantGotoInstructionsPass {
    pub fn new() -> Self {
        Self {
            address_remappings: HashMap::new(),
        }
    }

    pub fn find_remapped_address(&self, address: u16) -> u16 {
        match self.address_remappings.get(&address) {
            Some(remapped_address) => self.find_remapped_address(*remapped_address),
            None => address,
        }
    }
}

impl PostCompilationPass for RemoveRedundantGotoInstructionsPass {
    fn run_once(&mut self, instructions: &mut Vec<LogicInstruction>) -> PostCompilationPassResult {
        self.address_remappings.clear();
        for (index, instruction) in instructions.iter().enumerate() {
            let LogicInstruction::Goto(instruction) = instruction else {
                continue;
            };

            if index == instructions.len() - 1 {
                break;
            }

            let next_instruction = &instructions[index + 1];
            let goto_next_instruction = instruction.jump_address == next_instruction.address();
            let goto_same_address_as_next_instruction =
                if let LogicInstruction::Goto(next_instruction) = next_instruction {
                    next_instruction.jump_address == instruction.address
                } else {
                    false
                };

            if goto_next_instruction || goto_same_address_as_next_instruction {
                self.address_remappings
                    .insert(instruction.address, next_instruction.address());
            }
        }

        if self.address_remappings.is_empty() {
            return PostCompilationPassResult::Unchanged;
        }

        // delete all remapped instructions
        instructions
            .retain(|instruction| !self.address_remappings.contains_key(&instruction.address()));

        for instruction in instructions.iter_mut() {
            match instruction {
                LogicInstruction::Condition(instruction) => {
                    instruction.skip_address = self.find_remapped_address(instruction.skip_address);
                }
                LogicInstruction::Goto(instruction) => {
                    instruction.jump_address = self.find_remapped_address(instruction.jump_address);
                }
                _ => {}
            }
        }

        PostCompilationPassResult::Changed
    }
}

// AGI Studio compatibility: AGI Studio can't decompile conditionals that make criss crossing jumps
// This ends up generating _slightly_ larger resources (because of the extra GOTO statements that it
// inserts) but the resulting game can be decompiled in all the AGI IDEs
pub fn make_conditionals_self_contained(
    instructions: &mut Vec<LogicInstruction>,
) -> PostCompilationPassResult {
    let mut result = PostCompilationPassResult::Unchanged;

    let instruction_indexes = instructions
        .iter()
        .enumerate()
        .map(|(index, instruction)| (instruction.address(), index))
        .collect::<HashMap<_, _>>();
    let mut max_address = instruction_indexes.keys().max().copied();
    let mut block_end_indexes = VecDeque::from([instructions.len()]);
    let mut block_end_gotos: VecDeque<Option<LogicGoto>> = VecDeque::new();

    let new_instructions = instructions
        .iter()
        .enumerate()
        .flat_map(|(index, instruction)| {
            let mut block_ended = false;
            let mut output_instruction = instruction.clone();
            let mut this_block_end_gotos: Vec<LogicGoto> = vec![];

            while index > block_end_indexes[0] {
                block_end_indexes.pop_front();
                block_ended = true;
                if let Some(Some(block_end_goto)) = block_end_gotos.pop_front() {
                    this_block_end_gotos.push(block_end_goto);
                }
            }

            if let LogicInstruction::Condition(instruction) = instruction {
                let skip_index = instruction_indexes.get(&instruction.skip_address).unwrap();
                let containing_block_end = block_end_indexes[0];

                if *skip_index > containing_block_end {
                    let goto_address = max_address.map(|addr| addr + 1).unwrap_or(0);
                    max_address = Some(goto_address);
                    let goto = LogicGoto {
                        address: goto_address,
                        jump_address: instruction.skip_address,
                    };
                    output_instruction = LogicInstruction::Condition(LogicCondition {
                        skip_address: goto_address,
                        ..instruction.clone()
                    });
                    result = PostCompilationPassResult::Changed;
                    block_end_indexes.push_front(containing_block_end);
                    block_end_gotos.push_front(Some(goto));
                } else {
                    block_end_indexes.push_front(*skip_index);
                    block_end_gotos.push_front(None);
                }
            }

            if block_ended && this_block_end_gotos.len() > 0 {
                result = PostCompilationPassResult::Changed;
                Box::new(
                    this_block_end_gotos
                        .into_iter()
                        .map(LogicInstruction::Goto)
                        .chain(std::iter::once(output_instruction)),
                ) as Box<dyn Iterator<Item = LogicInstruction>>
            } else {
                Box::new(std::iter::once(output_instruction))
                    as Box<dyn Iterator<Item = LogicInstruction>>
            }
        })
        .collect::<Vec<_>>();

    if result == PostCompilationPassResult::Changed {
        *instructions = new_instructions;
    }

    result
}
