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

### 后端开发 (Rust)
```bash
# 运行后端服务
cd backend
cargo run

# 构建后端
cargo build

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

### 前端开发
```bash
# 开发环境尚未设置，预计使用现代前端框架
# 前端目录结构待实现
```

## 当前技术栈

### 后端
- **语言**: Rust
- **版本**: 2024 edition
- **构建工具**: Cargo
- **入口文件**: `backend/src/main.rs`

### 前端
- **状态**: 未实现
- **计划**: 多端支持(Web、小程序、原生应用)

## 开发规范

### 1. 代码组织
```
project-root/
├── backend/              # 后端服务 (Rust)
│   ├── Cargo.toml       # Rust项目配置
│   ├── src/
│   │   ├── main.rs      # 程序入口
│   │   ├── controllers/ # 控制器 (待实现)
│   │   ├── services/    # 业务逻辑 (待实现)
│   │   ├── models/      # 数据模型 (待实现)
│   │   ├── routes/      # 路由定义 (待实现)
│   │   └── utils/       # 工具函数 (待实现)
│   └── tests/          # 测试文件 (待实现)
├── frontend/            # 前端目录 (待实现)
├── web-admin/           # 管理端Web (待实现)
├── web-portal/          # 门户网站 (待实现)
├── mini-program-wx/     # 微信小程序 (待实现)
├── mini-program-alipay/ # 支付宝小程序 (待实现)
├── mobile-ios/          # iOS应用 (待实现)
├── mobile-android/      # Android应用 (待实现)
└── shared/             # 共享组件和工具 (待实现)
```

### 2. API设计原则
- RESTful风格
- 统一响应格式
- 版本控制（/api/v1/）
- JWT认证
- 请求限流

### 3. 安全要求
- HTTPS传输
- 敏感数据加密
- SQL注入防护
- XSS/CSRF防护
- 文件上传校验
- 权限细粒度控制

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

### Phase 1: MVP版本（2个月）
- [ ] 基础架构搭建
- [ ] 用户认证系统
- [ ] 管理端核心功能
- [ ] 医生端基础功能
- [ ] 患者端预约功能

### Phase 2: 功能完善（1个月）
- [ ] 视频问诊功能
- [ ] 直播功能集成
- [ ] 圈子社区功能
- [ ] 内容管理系统

### Phase 3: 移动端开发（1个月）
- [ ] 微信小程序
- [ ] 支付宝小程序
- [ ] 移动端适配优化

### Phase 4: 原生应用（后续）
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
