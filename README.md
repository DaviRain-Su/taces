# 香河香草中医诊所多端诊疗平台 (TCM Telemedicine Platform)

## 项目概述

香河香草中医诊所多端诊疗平台是一个综合性的中医医疗服务系统，通过数字化手段提升中医诊疗服务效率，为医生和患者提供便捷的线上服务。

### 核心功能
- 👨‍⚕️ **医生管理** - 医生资质认证、排班管理、在线接诊
- 👥 **患者服务** - 预约挂号、在线问诊、查看处方
- 💊 **处方管理** - 电子处方开具、中药配方管理
- 📱 **多端支持** - Web端、微信小程序、支付宝小程序、iOS/Android

## 快速开始

### 环境要求
- Docker & Docker Compose
- Rust 1.70+
- MySQL 8.0+

### 本地开发

1. **克隆仓库**
```bash
git clone https://github.com/your-org/tcm-telemedicine.git
cd tcm-telemedicine
```

2. **启动数据库**
```bash
make db-up
```

3. **配置环境变量**
```bash
cd backend
cp .env.example .env
# 编辑 .env 文件配置数据库连接等信息
```

4. **运行后端服务**
```bash
make dev
```

服务将在 http://localhost:3000 启动

5. **初始化测试数据**
```bash
make db-seed
```

### 测试账号
- 管理员: admin / admin123
- 医生: doctor_dong / doctor123
- 患者: patient1 / patient123

## 项目结构

```
.
├── backend/              # Rust 后端服务
│   ├── src/
│   │   ├── controllers/  # API 控制器
│   │   ├── services/     # 业务逻辑
│   │   ├── models/       # 数据模型
│   │   ├── routes/       # 路由定义
│   │   └── middleware/   # 中间件
│   ├── migrations/       # 数据库迁移
│   └── tests/           # 测试文件
├── frontend/            # 前端项目 (待实现)
├── docker-compose.yml   # Docker 配置
└── Makefile            # 便捷命令
```

## 开发指南

### 运行测试
```bash
# 运行所有测试
make test

# 运行单元测试
make test-unit

# 运行集成测试
make test-integration
```

### 代码规范
```bash
# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

### 数据库操作
```bash
# 重置数据库
make db-reset

# 查看数据库 (通过 Adminer)
open http://localhost:8080
```

## API 文档

完整的 API 文档请查看 [backend/README.md](backend/README.md)

主要接口包括：
- 认证接口 (`/api/v1/auth/*`)
- 用户管理 (`/api/v1/users/*`)
- 医生管理 (`/api/v1/doctors/*`)
- 预约管理 (`/api/v1/appointments/*`)
- 处方管理 (`/api/v1/prescriptions/*`)

## 技术栈

### 后端
- **语言**: Rust
- **框架**: Axum
- **数据库**: MySQL 8.0
- **认证**: JWT
- **ORM**: SQLx

### 前端 (计划中)
- **Web**: React + TypeScript
- **小程序**: Taro
- **移动端**: React Native

## 部署

### Docker 部署
```bash
# 构建镜像
docker build -t tcm-backend ./backend

# 运行容器
docker run -p 3000:3000 --env-file .env tcm-backend
```

### 生产环境配置
- 使用环境变量管理敏感信息
- 配置 HTTPS
- 设置数据库主从复制
- 配置日志收集和监控

## 贡献指南

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## 联系方式

- 项目负责人: 董老师
- 诊所地址: 香河香草中医诊所
- 技术支持: dev@tcm-clinic.com

---

**中药创新，推动 CHINESE MEDICINE 共赢健康未来** 🌿