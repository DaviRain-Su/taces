# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# 香河香草中医诊所多端诊疗平台开发指南 (TCM Telemedicine Platform)

## 项目概述

香河香草中医诊所多端诊疗平台是一个综合性的中医医疗服务系统，旨在通过数字化手段提升中医诊疗服务效率，为医生和患者提供便捷的线上服务。系统采用前后端分离架构，支持Web端、微信小程序、支付宝小程序以及iOS/Android原生应用。

### 核心理念
- **品牌定位**：中药创新，推动 CHINESE MEDICINE 共赢健康未来
- **服务宗旨**：充分发挥董老师30多年中医经验，结合现代信息技术，提供优质中医诊疗服务
- **技术策略**：渐进式开发，先实现Web端，后续扩展至各移动端平台

## 系统架构

### 技术架构
```
┌─────────────────────────────────────────────────────┐
│                   前端展示层                          │
├──────────┬──────────┬──────────┬──────────┬─────────┤
│  Web端   │ 微信小程序 │支付宝小程序│   iOS    │Android │
└──────────┴──────────┴──────────┴──────────┴─────────┘
                          │
                    ┌─────┴─────┐
                    │  API网关   │
                    └─────┬─────┘
                          │
┌─────────────────────────┴─────────────────────────────┐
│                    业务服务层                          │
├──────────┬──────────┬──────────┬──────────┬──────────┤
│用户服务  │预约服务  │问诊服务  │内容服务  │直播服务  │
└──────────┴──────────┴──────────┴──────────┴──────────┘
                          │
┌─────────────────────────┴─────────────────────────────┐
│                    数据存储层                          │
├──────────┬──────────┬──────────┬──────────┬──────────┤
│ MySQL    │ Redis    │ MongoDB  │   OSS    │Elasticsearch│
└──────────┴──────────┴──────────┴──────────┴──────────┘
```

### 用户角色
1. **管理员**：系统管理、医生账号管理、内容审核、数据统计
2. **医生**：接诊、开方、发布内容、直播、患者管理
3. **患者**：预约挂号、在线问诊、查看处方、健康咨询

## 功能模块详解

### 1. 管理端（Web）

#### 1.1 组织架构管理
- **用户管理**
  - 新增用户（必填：账号、姓名、密码、性别、手机号）
  - 批量操作（删除、导出）
  - 权限分配（授权角色、授权数据）
  - 账号状态管理

- **科室管理**
  - 科室信息维护（名称、编码、联系人、描述）
  - 科室编码规则：如 ZY001（中医科）、ZJTN002（针灸推拿科）

#### 1.2 业务管理
- **医生管理**
  - 医生信息展示（基本信息、职称、科室、联系方式）
  - 执业资质管理（执业证、职称证、身份证）

- **号源管理**
  - 挂号记录查询（按患者、日期、医生、状态筛选）
  - 就诊状态跟踪（已挂号、已就诊、已取消）

- **内容管理**
  - 新闻发布（标题、封面图310x298px、内容编辑）
  - 发布渠道（医院介绍、官网新闻、手机端、健康科普、活动动态）
  - 视频管理（支持MP4/AVI，≤30MB，建议1080P）
  - 直播管理（查看直播记录、二维码生成）

### 2. 医生端

#### 2.1 首页数据看板
- **实时数据**
  - 今日已接诊数
  - 今日已预约数
  - 今年总预约数
  - 今年总接诊数

#### 2.2 核心功能
- **今日问诊**
  - 分类查看：全部、待接诊、进行中、已完成
  - 患者信息快速浏览

- **预约管理**
  - 查看预约详情（时间、患者、病情描述）
  - 预约状态管理

- **内容发布**
  - 文章撰写（提供写作指南）
  - 直播预告发布
  - 视频问诊录制上传

- **患者管理**
  - 患者分组（最多5个分组）
  - 群发消息功能
  - 患者信息维护

- **个人中心**
  - 个人信息完善
  - 处方记录管理
  - 常用语/常用处方设置
  - 患者评价查看

### 3. 患者端

#### 3.1 首页功能
- 品牌展示
- 快捷入口：预约挂号、我的处方、直播、热门医生
- 医生推荐（展示医生简介、擅长领域）

#### 3.2 就医服务
- **预约挂号**
  - 选择科室医生
  - 选择就诊日期
  - 填写病情描述（100字以内）
  - 选择就诊方式

- **处方查询**
  - 处方列表（编号、就诊人、诊断、日期）
  - 处方详情查看
  - 历史用药追溯

- **视频问诊**
  - 一对一视频通话
  - 问诊记录保存

#### 3.3 互动功能
- **直播观看**
  - 直播预告查看
  - 直播提醒设置

- **圈子社区**
  - 加入感兴趣圈子（董老师、中医、慢性病等）
  - 发布内容（文字+图片，最多9张）
  - 敏感词自动过滤

#### 3.4 个人管理
- **就诊人管理**
  - 多就诊人信息维护
  - 设置默认就诊人

- **个人中心**
  - 预约记录
  - 处方记录
  - 消息中心

## 数据模型设计

### 核心实体
```javascript
// 用户基础信息
User {
  id: String,
  account: String,        // 账号
  name: String,          // 姓名
  password: String,      // 密码（加密存储）
  gender: String,        // 性别
  phone: String,         // 手机号
  email: String,         // 邮箱
  birthday: Date,        // 生日
  role: String,          // 角色：admin/doctor/patient
  status: String,        // 状态：active/inactive
  createdAt: Date,
  updatedAt: Date
}

// 医生信息
Doctor {
  userId: String,        // 关联用户ID
  certificateType: String,   // 证件类型
  idNumber: String,          // 身份证号
  hospital: String,          // 就职医院
  department: String,        // 科室
  title: String,            // 职称
  introduction: String,      // 个人简介
  specialties: Array,        // 擅长领域
  experience: String,        // 经历
  photos: {
    avatar: String,         // 医师照片
    license: String,        // 执业证照片
    idCardFront: String,    // 身份证正面
    idCardBack: String,     // 身份证反面
    titleCert: String       // 职称证书
  }
}

// 预约信息
Appointment {
  id: String,
  patientId: String,      // 患者ID
  doctorId: String,       // 医生ID
  appointmentDate: Date,  // 预约日期
  timeSlot: String,       // 时间段
  visitType: String,      // 就诊方式
  symptoms: String,       // 病情描述
  hasVisitedBefore: Boolean,  // 是否就诊过
  status: String,         // 状态：pending/confirmed/completed/cancelled
  createdAt: Date
}

// 处方信息
Prescription {
  id: String,
  code: String,           // 处方编号
  doctorId: String,       // 医生ID
  patientId: String,      // 患者ID
  patientName: String,    // 就诊人姓名
  diagnosis: String,      // 诊断结果
  medicines: Array,       // 药品列表
  instructions: String,   // 用药说明
  prescriptionDate: Date, // 开方日期
  createdAt: Date
}

// 直播信息
LiveStream {
  id: String,
  title: String,          // 直播主题
  hostId: String,         // 主播ID（医生）
  hostName: String,       // 主播姓名
  scheduledTime: Date,    // 直播时间
  streamUrl: String,      // 直播链接
  qrCode: String,         // 二维码
  status: String,         // 状态：scheduled/live/ended
  createdAt: Date
}

// 圈子帖子
CirclePost {
  id: String,
  authorId: String,       // 作者ID
  circleId: String,       // 圈子ID
  title: String,          // 主题
  content: String,        // 内容
  images: Array,          // 图片列表
  likes: Number,          // 点赞数
  comments: Number,       // 评论数
  status: String,         // 状态：active/deleted
  createdAt: Date,
  updatedAt: Date
}
```

## 开发环境设置

### 环境要求
- Rust 1.75+ (2024 edition)
- MySQL 8.0+
- Docker & Docker Compose
- Git

### 后端开发 (Rust)
```bash
# 克隆项目
git clone https://github.com/yourusername/taces.git
cd taces

# 启动数据库
docker-compose up -d

# 设置环境变量
cd backend
cp .env.example .env
# 编辑 .env 文件配置数据库连接

# 运行数据库迁移
cargo install sqlx-cli --no-default-features --features rustls,mysql
sqlx migrate run

# 导入测试数据
cargo run --bin seed

# 运行后端服务
cargo run

# 构建后端
cargo build --release

# 运行所有测试
cargo test -- --test-threads=1

# 运行单元测试
cargo test --test unit_tests

# 运行集成测试
cargo test --test integration_tests -- --test-threads=1

# 代码格式化
cargo fmt

# 代码检查
cargo clippy -- -D warnings

# 本地运行CI检查
../test-ci-locally.sh
```

### 数据库管理
```bash
# 访问MySQL
docker exec -it tcm_mysql mysql -utcm_user -ptcm_pass123 tcm_telemedicine

# 访问Adminer (Web数据库管理)
# 打开浏览器访问: http://localhost:8080
# 服务器: mysql
# 用户名: tcm_user
# 密码: tcm_pass123
# 数据库: tcm_telemedicine
```

### 前端开发
```bash
# 开发环境尚未设置，预计使用以下技术栈：
# - 管理端: React + Ant Design Pro
# - 患者端Web: Next.js + Tailwind CSS
# - 微信小程序: 原生小程序 / Taro
# - 支付宝小程序: 原生小程序 / Taro
# - iOS: Swift / React Native
# - Android: Kotlin / React Native
```

## 当前技术栈

### 后端（已实现）
- **语言**: Rust (2024 edition)
- **Web框架**: Axum 0.7
- **数据库**: MySQL 8.0 + SQLx
- **认证**: JWT (jsonwebtoken)
- **密码加密**: bcrypt
- **验证**: validator
- **序列化**: serde + serde_json
- **异步运行时**: Tokio
- **日志**: tracing
- **测试**: 单元测试 + 集成测试
- **CI/CD**: GitHub Actions

### 前端
- **状态**: 未实现
- **计划**: 多端支持(Web、小程序、原生应用)

## 开发规范

### 1. 代码组织
```
taces/
├── .github/
│   └── workflows/       # GitHub Actions CI/CD
│       ├── rust.yml     # 主分支CI
│       └── backend-ci.yml # 后端完整CI
├── backend/             # 后端服务 (Rust) ✅
│   ├── Cargo.toml      # Rust项目配置
│   ├── Cargo.lock      # 依赖锁定文件
│   ├── .env            # 环境变量配置
│   ├── .env.example    # 环境变量示例
│   ├── migrations/     # 数据库迁移文件
│   │   └── *.sql
│   ├── sql/            # SQL脚本
│   ├── src/
│   │   ├── main.rs     # 程序入口
│   │   ├── lib.rs      # 库入口
│   │   ├── bin/        # 可执行文件
│   │   │   └── seed.rs # 数据库种子
│   │   ├── config/     # 配置模块
│   │   │   ├── mod.rs
│   │   │   └── database.rs
│   │   ├── controllers/ # 控制器层 ✅
│   │   │   ├── mod.rs
│   │   │   ├── auth_controller.rs
│   │   │   ├── user_controller.rs
│   │   │   ├── doctor_controller.rs
│   │   │   ├── appointment_controller.rs
│   │   │   ├── prescription_controller.rs
│   │   │   ├── department_controller.rs
│   │   │   ├── patient_group_controller.rs
│   │   │   ├── patient_profile_controller.rs
│   │   │   ├── content_controller.rs
│   │   │   ├── live_stream_controller.rs
│   │   │   ├── circle_controller.rs
│   │   │   ├── circle_post_controller.rs
│   │   │   └── template_controller.rs
│   │   ├── services/   # 业务逻辑层 ✅
│   │   │   ├── mod.rs
│   │   │   ├── auth_service.rs
│   │   │   ├── user_service.rs
│   │   │   ├── doctor_service.rs
│   │   │   ├── appointment_service.rs
│   │   │   ├── prescription_service.rs
│   │   │   ├── department_service.rs
│   │   │   ├── patient_group_service.rs
│   │   │   ├── patient_profile_service.rs
│   │   │   ├── content_service.rs
│   │   │   ├── live_stream_service.rs
│   │   │   ├── circle_service.rs
│   │   │   ├── circle_post_service.rs
│   │   │   └── template_service.rs
│   │   ├── models/     # 数据模型 ✅
│   │   │   ├── mod.rs
│   │   │   ├── user.rs
│   │   │   ├── doctor.rs
│   │   │   ├── appointment.rs
│   │   │   ├── prescription.rs
│   │   │   ├── department.rs
│   │   │   ├── patient_group.rs
│   │   │   ├── patient_profile.rs
│   │   │   ├── content.rs
│   │   │   ├── live_stream.rs
│   │   │   ├── circle.rs
│   │   │   ├── circle_post.rs
│   │   │   └── template.rs
│   │   ├── routes/     # 路由定义 ✅
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── user.rs
│   │   │   ├── doctor.rs
│   │   │   ├── appointment.rs
│   │   │   ├── prescription.rs
│   │   │   ├── department.rs
│   │   │   ├── patient_group.rs
│   │   │   ├── patient_profile.rs
│   │   │   ├── content.rs
│   │   │   ├── live_stream.rs
│   │   │   ├── circle.rs
│   │   │   ├── circle_post.rs
│   │   │   └── template.rs
│   │   ├── middleware/ # 中间件 ✅
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   └── jwt_config.rs
│   │   └── utils/      # 工具函数 ✅
│   │       ├── mod.rs
│   │       ├── jwt.rs
│   │       ├── password.rs
│   │       └── test_helpers.rs
│   └── tests/          # 测试文件 ✅
│       ├── common/
│       │   └── mod.rs
│       ├── integration/
│       │   ├── mod.rs
│       │   ├── test_auth.rs
│       │   ├── test_user.rs
│       │   ├── test_doctor.rs
│       │   ├── test_appointment.rs
│       │   ├── test_prescription.rs
│       │   ├── test_department.rs
│       │   ├── test_patient_group.rs
│       │   ├── test_patient_profile.rs
│       │   ├── test_content.rs
│       │   ├── test_live_stream.rs
│       │   ├── test_circle.rs
│       │   ├── test_circle_post.rs
│       │   └── test_template.rs
│       └── unit/
│           ├── mod.rs
│           ├── test_jwt.rs
│           └── test_password.rs
├── docker-compose.yml   # Docker配置
├── test-ci-locally.sh  # 本地CI测试脚本
├── README.md           # 项目说明
├── CLAUDE.md           # Claude AI开发指南
├── frontend/           # 前端目录 (待实现)
├── web-admin/          # 管理端Web (待实现)
├── web-portal/         # 门户网站 (待实现)
├── mini-program-wx/    # 微信小程序 (待实现)
├── mini-program-alipay/# 支付宝小程序 (待实现)
├── mobile-ios/         # iOS应用 (待实现)
├── mobile-android/     # Android应用 (待实现)
└── shared/            # 共享组件和工具 (待实现)
```

### 2. API设计原则
- RESTful风格
- 统一响应格式
- 版本控制（/api/v1/）
- JWT认证
- 请求限流

### 3. API端点列表

#### 认证相关 ✅
- `POST /api/v1/auth/register` - 用户注册 ✅
- `POST /api/v1/auth/login` - 用户登录 ✅

#### 用户管理 ✅
- `GET /api/v1/users` - 获取用户列表（管理员）✅
- `GET /api/v1/users/:id` - 获取用户详情 ✅
- `PUT /api/v1/users/:id` - 更新用户信息 ✅
- `DELETE /api/v1/users/:id` - 删除用户（管理员）✅
- `DELETE /api/v1/users/batch/delete` - 批量删除用户（管理员）✅
- `GET /api/v1/users/batch/export` - 导出用户（管理员）✅

#### 医生管理 ✅
- `GET /api/v1/doctors` - 获取医生列表（公开）✅
- `GET /api/v1/doctors/:id` - 获取医生详情（公开）✅
- `POST /api/v1/doctors` - 创建医生档案（管理员）✅
- `PUT /api/v1/doctors/:id` - 更新医生信息 ✅
- `PUT /api/v1/doctors/:id/photos` - 更新医生照片 ✅
- `GET /api/v1/doctors/by-user/:user_id` - 根据用户ID获取医生信息 ✅

#### 科室管理 ✅
- `GET /api/v1/departments` - 获取科室列表 ✅
- `GET /api/v1/departments/:id` - 获取科室详情 ✅
- `GET /api/v1/departments/code/:code` - 根据编码获取科室 ✅
- `POST /api/v1/departments` - 创建科室（管理员）✅
- `PUT /api/v1/departments/:id` - 更新科室信息（管理员）✅
- `DELETE /api/v1/departments/:id` - 删除科室（管理员）✅

#### 预约管理 ✅
- `GET /api/v1/appointments` - 获取预约列表 ✅
- `GET /api/v1/appointments/:id` - 获取预约详情 ✅
- `POST /api/v1/appointments` - 创建预约 ✅
- `PUT /api/v1/appointments/:id` - 更新预约 ✅
- `PUT /api/v1/appointments/:id/cancel` - 取消预约 ✅
- `GET /api/v1/appointments/doctor/:doctor_id` - 获取医生的预约 ✅
- `GET /api/v1/appointments/patient/:patient_id` - 获取患者的预约 ✅
- `GET /api/v1/appointments/available-slots` - 获取可用时间段 ✅

#### 处方管理 ✅
- `GET /api/v1/prescriptions` - 获取处方列表（管理员）✅
- `GET /api/v1/prescriptions/:id` - 获取处方详情 ✅
- `POST /api/v1/prescriptions` - 创建处方（医生）✅
- `GET /api/v1/prescriptions/code/:code` - 根据处方编号获取处方 ✅
- `GET /api/v1/prescriptions/patient/:patient_id` - 获取患者处方 ✅
- `GET /api/v1/prescriptions/doctor/:doctor_id` - 获取医生开具的处方 ✅

#### 患者分组管理 ✅
- `GET /api/v1/patient-groups` - 获取医生的患者分组列表 ✅
- `GET /api/v1/patient-groups/:id` - 获取分组详情 ✅
- `POST /api/v1/patient-groups` - 创建患者分组（医生）✅
- `PUT /api/v1/patient-groups/:id` - 更新分组信息 ✅
- `DELETE /api/v1/patient-groups/:id` - 删除分组 ✅
- `POST /api/v1/patient-groups/:id/members` - 添加分组成员 ✅
- `DELETE /api/v1/patient-groups/:id/members` - 移除分组成员 ✅
- `POST /api/v1/patient-groups/:id/message` - 发送群消息 ✅

#### 就诊人管理 ✅
- `GET /api/v1/patient-profiles` - 获取就诊人列表 ✅
- `GET /api/v1/patient-profiles/:id` - 获取就诊人详情 ✅
- `POST /api/v1/patient-profiles` - 创建就诊人 ✅
- `PUT /api/v1/patient-profiles/:id` - 更新就诊人信息 ✅
- `DELETE /api/v1/patient-profiles/:id` - 删除就诊人 ✅
- `PUT /api/v1/patient-profiles/:id/default` - 设置默认就诊人 ✅

#### 内容管理 ✅
- `GET /api/v1/content/articles` - 获取文章列表 ✅
- `GET /api/v1/content/articles/:id` - 获取文章详情 ✅
- `POST /api/v1/content/articles` - 创建文章（医生/管理员）✅
- `PUT /api/v1/content/articles/:id` - 更新文章 ✅
- `POST /api/v1/content/articles/:id/publish` - 发布文章 ✅
- `POST /api/v1/content/articles/:id/unpublish` - 取消发布文章 ✅
- `DELETE /api/v1/content/articles/:id` - 删除文章 ✅
- `GET /api/v1/content/videos` - 获取视频列表 ✅
- `GET /api/v1/content/videos/:id` - 获取视频详情 ✅
- `POST /api/v1/content/videos` - 创建视频（医生/管理员）✅
- `PUT /api/v1/content/videos/:id` - 更新视频 ✅
- `POST /api/v1/content/videos/:id/publish` - 发布视频 ✅
- `DELETE /api/v1/content/videos/:id` - 删除视频 ✅
- `GET /api/v1/content/categories` - 获取分类列表 ✅
- `POST /api/v1/content/categories` - 创建分类（管理员）✅

#### 直播管理 ✅
- `GET /api/v1/live-streams` - 获取直播列表 ✅
- `GET /api/v1/live-streams/:id` - 获取直播详情 ✅
- `POST /api/v1/live-streams` - 创建直播（医生）✅
- `PUT /api/v1/live-streams/:id` - 更新直播信息 ✅
- `DELETE /api/v1/live-streams/:id` - 删除直播 ✅
- `PUT /api/v1/live-streams/:id/start` - 开始直播 ✅
- `PUT /api/v1/live-streams/:id/end` - 结束直播 ✅
- `GET /api/v1/live-streams/upcoming` - 获取即将开始的直播 ✅
- `GET /api/v1/live-streams/my` - 获取我的直播（医生）✅

#### 圈子管理 ✅
- `GET /api/v1/circles` - 获取圈子列表 ✅
- `GET /api/v1/circles/:id` - 获取圈子详情 ✅
- `POST /api/v1/circles` - 创建圈子 ✅
- `PUT /api/v1/circles/:id` - 更新圈子信息 ✅
- `DELETE /api/v1/circles/:id` - 删除圈子 ✅
- `POST /api/v1/circles/:id/join` - 加入圈子 ✅
- `POST /api/v1/circles/:id/leave` - 退出圈子 ✅
- `GET /api/v1/circles/:id/members` - 获取圈子成员 ✅
- `PUT /api/v1/circles/:id/members/:user_id/role` - 更新成员角色 ✅
- `DELETE /api/v1/circles/:id/members/:user_id` - 移除成员 ✅
- `GET /api/v1/my-circles` - 获取我加入的圈子 ✅

#### 圈子帖子管理 ✅
- `GET /api/v1/posts` - 获取帖子列表 ✅
- `GET /api/v1/posts/:id` - 获取帖子详情 ✅
- `POST /api/v1/posts` - 发布帖子（圈子成员）✅
- `PUT /api/v1/posts/:id` - 更新帖子（作者）✅
- `DELETE /api/v1/posts/:id` - 删除帖子（作者/管理员）✅
- `GET /api/v1/users/:user_id/posts` - 获取用户的帖子 ✅
- `GET /api/v1/circles/:circle_id/posts` - 获取圈子的帖子 ✅
- `POST /api/v1/posts/:id/like` - 点赞/取消点赞 ✅
- `GET /api/v1/posts/:id/comments` - 获取帖子评论 ✅
- `POST /api/v1/posts/:id/comments` - 发表评论 ✅
- `DELETE /api/v1/comments/:id` - 删除评论（作者/管理员）✅

#### 常用语和处方模板管理 ✅
- `GET /api/v1/templates/common-phrases` - 获取常用语列表（医生）✅
- `GET /api/v1/templates/common-phrases/:id` - 获取常用语详情（医生）✅
- `POST /api/v1/templates/common-phrases` - 创建常用语（医生）✅
- `PUT /api/v1/templates/common-phrases/:id` - 更新常用语（医生）✅
- `DELETE /api/v1/templates/common-phrases/:id` - 删除常用语（医生）✅
- `POST /api/v1/templates/common-phrases/:id/use` - 增加使用次数（医生）✅
- `GET /api/v1/templates/prescription-templates` - 获取处方模板列表（医生）✅
- `GET /api/v1/templates/prescription-templates/:id` - 获取处方模板详情（医生）✅
- `POST /api/v1/templates/prescription-templates` - 创建处方模板（医生）✅
- `PUT /api/v1/templates/prescription-templates/:id` - 更新处方模板（医生）✅
- `DELETE /api/v1/templates/prescription-templates/:id` - 删除处方模板（医生）✅
- `POST /api/v1/templates/prescription-templates/:id/use` - 增加使用次数（医生）✅

#### 患者评价系统 ✅
- `GET /api/v1/reviews` - 获取评价列表（管理员）✅
- `GET /api/v1/reviews/:id` - 获取评价详情 ✅
- `POST /api/v1/reviews` - 创建评价（患者）✅
- `PUT /api/v1/reviews/:id` - 更新评价（作者）✅
- `POST /api/v1/reviews/:id/reply` - 回复评价（医生）✅
- `PUT /api/v1/reviews/:id/visibility` - 更新评价可见性（管理员）✅
- `GET /api/v1/reviews/doctor/:doctor_id/reviews` - 获取医生评价（公开）✅
- `GET /api/v1/reviews/doctor/:doctor_id/statistics` - 获取医生评价统计（公开）✅
- `GET /api/v1/reviews/patient/:patient_id/reviews` - 获取患者评价 ✅
- `GET /api/v1/reviews/tags` - 获取评价标签（公开）✅
- `POST /api/v1/reviews/tags` - 创建评价标签（管理员）✅

#### 通知系统 ✅
- `GET /api/v1/notifications` - 获取用户通知列表 ✅
- `GET /api/v1/notifications/:id` - 获取通知详情 ✅
- `PUT /api/v1/notifications/:id/read` - 标记为已读 ✅
- `PUT /api/v1/notifications/read-all` - 全部标记已读 ✅
- `DELETE /api/v1/notifications/:id` - 删除通知 ✅
- `GET /api/v1/notifications/stats` - 获取通知统计 ✅
- `GET /api/v1/notifications/settings` - 获取通知设置 ✅
- `PUT /api/v1/notifications/settings` - 更新通知设置 ✅
- `POST /api/v1/notifications/push-token` - 注册推送token ✅
- `POST /api/v1/notifications/announcement` - 发送系统公告（管理员）✅

#### 统计分析 ✅
- `GET /api/v1/statistics/dashboard` - 管理员仪表盘（管理员）✅
- `GET /api/v1/statistics/doctor/:doctor_id` - 医生统计 ✅
- `GET /api/v1/statistics/patient` - 患者统计 ✅
- `GET /api/v1/statistics/departments` - 科室统计（公开）✅
- `GET /api/v1/statistics/top-doctors` - 热门医生（公开）✅
- `GET /api/v1/statistics/top-content` - 热门内容（公开）✅
- `GET /api/v1/statistics/appointment-trends` - 预约趋势（管理员）✅
- `GET /api/v1/statistics/time-slots` - 时间段分布（管理员）✅
- `GET /api/v1/statistics/content` - 内容统计（管理员）✅
- `GET /api/v1/statistics/live-streams` - 直播统计（管理员）✅
- `GET /api/v1/statistics/circles` - 圈子统计（管理员）✅
- `GET /api/v1/statistics/user-growth` - 用户增长（管理员）✅
- `GET /api/v1/statistics/appointment-heatmap` - 预约热力图（管理员）✅
- `GET /api/v1/statistics/export` - 数据导出（管理员）✅

#### 待实现接口
- 支付相关
  - 微信支付集成
  - 支付宝支付集成
  - 订单管理
  - 退款处理
- 视频问诊
  - WebRTC信令服务
  - 视频会话管理
  - 问诊记录存储
  - 录制功能

### 4. 安全要求
- HTTPS传输
- 敏感数据加密（bcrypt密码加密）
- SQL注入防护（使用参数化查询）
- XSS/CSRF防护
- 文件上传校验
- 权限细粒度控制（基于角色的访问控制）
- JWT令牌认证
- 环境变量管理敏感配置

## 部署方案

### 1. 基础设施
- 云服务商：阿里云/腾讯云
- 容器化部署：Docker + Kubernetes
- 负载均衡：Nginx
- CDN加速：静态资源分发

### 2. 数据库架构
- 主从复制
- 读写分离
- 定期备份
- 灾难恢复

### 3. 监控告警
- 应用性能监控
- 错误日志收集
- 业务指标监控
- 实时告警通知

## 开发里程碑

### Phase 1: MVP版本（2个月）✅ 已完成
- [x] 基础架构搭建
  - [x] Rust + Axum 框架搭建
  - [x] MySQL 数据库集成
  - [x] Docker 容器化配置
  - [x] CI/CD 流程配置
- [x] 用户认证系统（JWT）
  - [x] 用户注册/登录
  - [x] JWT 令牌生成和验证
  - [x] 基于角色的权限控制
- [x] 管理端核心功能
  - [x] 用户管理（增删改查）
  - [x] 批量操作（删除、导出）
  - [x] 科室管理
- [x] 医生端基础功能
  - [x] 医生档案管理
  - [x] 资质认证信息
  - [x] 医生照片管理
- [x] 患者端核心功能
  - [x] 预约挂号功能
  - [x] 处方查询功能
  - [x] 就诊人管理（家庭成员）
- [x] 新增功能
  - [x] 患者分组管理（医生端）
  - [x] 群发消息功能

### Phase 2: 功能完善（大部分已完成）
- [x] 内容管理系统 ✅
  - [x] 文章管理（Article）
  - [x] 视频管理（Video）
  - [x] 内容分类管理
  - [x] 发布渠道管理
  - [x] 浏览量统计
- [x] 直播功能集成 ✅
  - [x] 直播创建和管理
  - [x] 直播状态转换（预告/直播中/已结束）
  - [x] 直播二维码生成
  - [x] 我的直播列表
  - [x] 即将开始的直播
- [x] 圈子社区功能 ✅
  - [x] 圈子创建和管理（Circle）
  - [x] 成员管理（加入/退出/角色管理）
  - [x] 帖子发布（CirclePost）
  - [x] 互动功能（点赞、评论）
  - [x] 敏感词过滤
  - [x] 分页查询
- [ ] 视频问诊功能
  - [ ] WebRTC 集成
  - [ ] 问诊记录保存

### Phase 3: 系统完善（大部分已完成）
- [x] 常用语和处方模板 ✅
  - [x] 常用语管理（诊断、医嘱、症状分类）
  - [x] 处方模板管理
  - [x] 使用次数统计
  - [x] 权限控制（仅医生可用）
- [x] 患者评价系统 ✅
  - [x] 评价创建和管理
  - [x] 医生回复功能
  - [x] 评分统计
  - [x] 标签系统
- [x] 通知系统 ✅
  - [x] 站内通知
  - [x] 通知设置管理
  - [x] 推送token管理
  - [x] 系统公告
  - [x] 短信/邮件日志
  - [x] 批量通知
- [x] 统计分析功能 ✅
  - [x] 管理员仪表盘
  - [x] 医生统计
  - [x] 患者统计
  - [x] 预约趋势分析
  - [x] 科室统计
  - [x] 内容热度统计
  - [x] 用户增长分析
  - [x] 数据导出（CSV）
- [ ] 支付系统集成
  - [ ] 微信支付
  - [ ] 支付宝支付

### Phase 4: 前端开发（待启动）
- [ ] 管理端 Web（React + Ant Design Pro）
- [ ] 患者端 Web（Next.js）
- [ ] 微信小程序
- [ ] 支付宝小程序
- [ ] 移动端适配优化

### Phase 5: 原生应用（后续）
- [ ] iOS应用开发
- [ ] Android应用开发
- [ ] 应用商店发布

## 注意事项

1. **医疗合规**
   - 确保符合医疗行业相关法规
   - 保护患者隐私信息
   - 医生资质严格审核

2. **用户体验**
   - 界面简洁易用，适合不同年龄段用户
   - 响应速度优化
   - 错误提示友好

3. **数据安全**
   - 患者信息加密存储
   - 定期安全审计
   - 访问日志记录

4. **扩展性考虑**
   - 模块化设计
   - 微服务架构
   - 接口标准化

## 第三方服务集成

1. **视频通话**：声网Agora/腾讯实时音视频
2. **直播服务**：快手直播SDK（已确定）
3. **短信服务**：阿里云短信
4. **支付接口**：微信支付/支付宝
5. **云存储**：阿里云OSS/腾讯云COS
6. **推送服务**：极光推送/个推

## 开发团队建议

- 项目经理：1名
- 后端开发：2-3名
- 前端开发：2名
- 移动端开发：2名
- UI设计师：1名
- 测试工程师：1名
- 运维工程师：1名

## 项目预算考虑

1. **开发成本**：人力成本（4个月）
2. **服务器成本**：云服务器、带宽、存储
3. **第三方服务**：视频通话、直播、短信等
4. **运营成本**：SSL证书、域名、应用市场费用

---

*本指南将随项目进展持续更新，确保开发团队始终拥有最新的技术规范和业务需求。*

## 常见问题

### Q: 如何运行测试时避免数据库连接错误？
A: 确保Docker容器正在运行，并且已经执行了数据库迁移。测试使用端口3307的测试数据库。

### Q: JWT认证失败怎么办？
A: 确保在测试和运行时设置了正确的JWT_SECRET环境变量。

### Q: 如何添加新的API端点？
A: 
1. 在 `models/` 中定义数据模型
2. 在 `services/` 中实现业务逻辑
3. 在 `controllers/` 中创建控制器
4. 在 `routes/` 中注册路由
5. 添加相应的测试

### Q: 如何处理数据库迁移？
A: 使用SQLx CLI工具：
```bash
# 创建新迁移
sqlx migrate add <migration_name>

# 运行迁移
sqlx migrate run

# 回滚迁移
sqlx migrate revert
```

## 测试账号

开发环境默认测试账号（通过 `cargo run --bin seed` 创建）：

- **管理员**: admin / admin123
- **医生**: doctor_dong / doctor123
- **医生**: doctor_wang / doctor123  
- **患者**: patient1-10 / patient123

## 联系方式

- 项目负责人: [待定]
- 技术支持: [待定]
- 问题反馈: 请在GitHub Issues中提交
