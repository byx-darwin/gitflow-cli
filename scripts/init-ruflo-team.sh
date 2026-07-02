#!/bin/bash

echo "🚀 初始化 Ruflo 智能体团队..."

# 初始化 swarm
npx ruflo@latest swarm init --topology hierarchical-mesh --max-agents 15 --strategy specialized

sleep 2

# 📝 创建智能体成员（使用实际支持的类型）

# 1. 协调者（替代 queen-coordinator 和 planner）
npx ruflo@latest agent spawn -t coordinator --name master-coordinator

# 2. 研究员
npx ruflo@latest agent spawn -t researcher --name prior-art-researcher

# 3. 架构师
npx ruflo@latest agent spawn -t architect --name rust-architect

# 4. 核心架构师（更专业）
npx ruflo@latest agent spawn -t core-architect --name system-architect

# 5. 开发者（用 coder 替代 core-coder 和 backend-dev）
npx ruflo@latest agent spawn -t coder --name core-coder
npx ruflo@latest agent spawn -t coder --name app-coder

# 6. 测试工程师
npx ruflo@latest agent spawn -t tester --name test-engineer

# 7. 测试架构师
npx ruflo@latest agent spawn -t test-architect --name test-arch

# 8. 审查员
npx ruflo@latest agent spawn -t reviewer --name code-reviewer

# 9. 安全架构师
npx ruflo@latest agent spawn -t security-architect --name security-reviewer

# 10. 安全审计员
npx ruflo@latest agent spawn -t security-auditor --name security-auditor

# 11. 内存专家
npx ruflo@latest agent spawn -t memory-specialist --name memory-curator

# 12. 性能工程师（替代 perf-analyzer）
npx ruflo@latest agent spawn -t performance-engineer --name performance-reviewer

# 13. 优化师
npx ruflo@latest agent spawn -t optimizer --name code-optimizer

# 14. 分析师
npx ruflo@latest agent spawn -t analyst --name requirement-analyst

# 15. Swarm 专家
npx ruflo@latest agent spawn -t swarm-specialist --name swarm-coordinator

echo "✅ 团队创建完成！共 15 个智能体已就位。"
