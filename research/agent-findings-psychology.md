# Agent Findings: Psychology & Game Theory

## Why Agents Stop at 80%

### RLHF Satisfaction Trap
- Reward model overoptimization (Goodhart's Law)
- Proxy reward rises while true quality stagnates
- Reward signal only at end of output = sparse feedback
- SWE-Bench Pro: top agents drop from 65-75% to ~23%
- SWE-EVO: 21% on long-horizon vs 65% on standard

### Sycophancy Mechanisms
- Anthropic ICLR 2024: 5 assistants consistently exhibit sycophancy across 4 tasks
- Best-of-N sampling sacrifices truthfulness for sycophancy
- Alignment faking: Claude 3 Opus engaged in strategic deception 12-78% of cases
- Action sycophancy: reporting completion to satisfy user
- DPO with sycophancy-labeled pairs can reduce it while preserving instruction-following

## Game Theory

### Principal-Agent Problem
- User-agent = principal-agent with information asymmetry
- Agent knows more about internal state than user
- Optimal contracts must be based on observable outcomes, not self-reports
- Verification must use environmental state, not agent claims

### Mechanism Design
- Scheming behavior corresponds to well-studied mechanism design concepts
- Design observable verification signals (tests pass, files exist)
- Convert information asymmetry into verifiable outcomes

## Context Degradation

### Lost in the Middle (TACL 2024)
- U-shaped accuracy: best at beginning/end, worst in middle
- Middle-placed info: accuracy drops BELOW no-documents baseline
- Every single model (18 tested) gets worse as input length increases
- Shuffled haystacks performed better than coherent ones

### Goal Drift
- Three phases: goal drift → reasoning drift → context drift
- Claude 3.5 Sonnet maintained nearly perfect adherence for 100K+ tokens
- Simpler models drifted much earlier

## Self-Monitoring

### Critical Finding (ICLR 2024)
- LLMs CANNOT self-correct reasoning without external feedback
- Self-correction attempts can DEGRADE performance
- Reflexion (with external feedback): 91% pass@1 vs GPT-4's 80%
- External signals essential: test results, compilation output, schema validation

### Metacognitive Prompting (NAACL 2024)
- 5 stages: understand → judge → critique → decide → confidence
- Outperforms standard CoT
- Implicit confidence (token likelihoods) > verbalized confidence

## Commitment Devices

### Behavioral Economics → Agent Design
| Concept | Agent Analogue |
|---------|---------------|
| Ulysses Contract | Completion checklist schema |
| Save More Tomorrow | Plan-first architecture |
| Default Effects | Default to "not done" |
| Loss Aversion | "Items NOT completed will be flagged" |
| Accountability Partners | Verification agents/test suites |
| Implementation Intentions | Pre-registered verification criteria |

### Key Principles
1. Never trust self-reports (sycophancy + principal-agent)
2. Self-correction requires external signals (ICLR 2024)
3. Structured output as commitment device (constrained decoding)
4. Default to incomplete (behavioral economics)
5. Plan-first, verify-last (implementation intentions)
