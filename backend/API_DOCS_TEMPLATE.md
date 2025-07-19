# 常用语和处方模板 API 文档

## 概述
常用语和处方模板功能是专门为医生设计的工具，帮助医生快速使用预定义的诊断用语和处方配置，提高工作效率。

## 功能特点
- **常用语管理**：支持诊断、医嘱、症状描述三种分类
- **处方模板管理**：预定义完整的处方配置，包括药品列表
- **使用统计**：记录使用次数，常用项目自动排前
- **权限控制**：仅医生可以使用，每位医生只能管理自己的模板

## API 端点

### 常用语管理

#### 1. 创建常用语
```http
POST /api/v1/templates/common-phrases
Authorization: Bearer <doctor_token>
Content-Type: application/json

{
  "category": "diagnosis",  // 分类：diagnosis(诊断), advice(医嘱), symptom(症状描述)
  "content": "风寒感冒，症见恶寒重、发热轻、无汗、头痛、肢节酸疼"
}

Response 200:
{
  "success": true,
  "message": "Common phrase created successfully",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "doctor_id": "doctor-uuid",
    "category": "diagnosis",
    "content": "风寒感冒，症见恶寒重、发热轻、无汗、头痛、肢节酸疼",
    "usage_count": 0,
    "is_active": true,
    "created_at": "2024-01-05T12:00:00Z",
    "updated_at": "2024-01-05T12:00:00Z"
  }
}
```

#### 2. 获取常用语列表
```http
GET /api/v1/templates/common-phrases?category=diagnosis&search=感冒&page=1&page_size=10
Authorization: Bearer <doctor_token>

Response 200:
{
  "success": true,
  "message": "Common phrases retrieved successfully",
  "data": {
    "phrases": [
      {
        "id": "phrase-uuid",
        "doctor_id": "doctor-uuid",
        "category": "diagnosis",
        "content": "风寒感冒，症见恶寒重、发热轻",
        "usage_count": 15,
        "is_active": true,
        "created_at": "2024-01-05T12:00:00Z",
        "updated_at": "2024-01-05T12:00:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "page_size": 10,
      "total": 25,
      "total_pages": 3
    }
  }
}
```

#### 3. 获取常用语详情
```http
GET /api/v1/templates/common-phrases/:id
Authorization: Bearer <doctor_token>

Response 200:
{
  "success": true,
  "message": "Common phrase retrieved successfully",
  "data": { /* 常用语详情 */ }
}
```

#### 4. 更新常用语
```http
PUT /api/v1/templates/common-phrases/:id
Authorization: Bearer <doctor_token>
Content-Type: application/json

{
  "content": "更新后的诊断描述",  // 可选
  "is_active": true              // 可选
}

Response 200:
{
  "success": true,
  "message": "Common phrase updated successfully",
  "data": { /* 更新后的常用语 */ }
}
```

#### 5. 删除常用语
```http
DELETE /api/v1/templates/common-phrases/:id
Authorization: Bearer <doctor_token>

Response 200:
{
  "success": true,
  "message": "Common phrase deleted successfully",
  "data": null
}
```

#### 6. 使用常用语（增加计数）
```http
POST /api/v1/templates/common-phrases/:id/use
Authorization: Bearer <doctor_token>

Response 200:
{
  "success": true,
  "message": "Usage count updated successfully",
  "data": null
}
```

### 处方模板管理

#### 1. 创建处方模板
```http
POST /api/v1/templates/prescription-templates
Authorization: Bearer <doctor_token>
Content-Type: application/json

{
  "name": "感冒清热颗粒方",
  "description": "用于风寒感冒引起的头痛发热",  // 可选
  "diagnosis": "风寒感冒",
  "medicines": [
    {
      "name": "感冒清热颗粒",
      "specification": "12g*10袋",
      "dosage": "1袋",
      "frequency": "一日3次",
      "duration": "3天",
      "usage": "开水冲服"
    },
    {
      "name": "板蓝根颗粒",
      "specification": "10g*20袋",
      "dosage": "1袋",
      "frequency": "一日3次",
      "duration": "3天",
      "usage": "开水冲服"
    }
  ],
  "instructions": "饭后服用，服药期间多饮水，注意休息"
}

Response 200:
{
  "success": true,
  "message": "Prescription template created successfully",
  "data": {
    "id": "template-uuid",
    "doctor_id": "doctor-uuid",
    "name": "感冒清热颗粒方",
    "description": "用于风寒感冒引起的头痛发热",
    "diagnosis": "风寒感冒",
    "medicines": [ /* 药品列表 */ ],
    "instructions": "饭后服用，服药期间多饮水，注意休息",
    "usage_count": 0,
    "is_active": true,
    "created_at": "2024-01-05T12:00:00Z",
    "updated_at": "2024-01-05T12:00:00Z"
  }
}
```

#### 2. 获取处方模板列表
```http
GET /api/v1/templates/prescription-templates?search=感冒&page=1&page_size=10
Authorization: Bearer <doctor_token>

Response 200:
{
  "success": true,
  "message": "Prescription templates retrieved successfully",
  "data": {
    "templates": [
      {
        "id": "template-uuid",
        "doctor_id": "doctor-uuid",
        "name": "感冒清热颗粒方",
        "description": "用于风寒感冒",
        "diagnosis": "风寒感冒",
        "medicines": [ /* 药品列表 */ ],
        "instructions": "饭后服用",
        "usage_count": 25,
        "is_active": true,
        "created_at": "2024-01-05T12:00:00Z",
        "updated_at": "2024-01-05T12:00:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "page_size": 10,
      "total": 15,
      "total_pages": 2
    }
  }
}
```

#### 3. 获取处方模板详情
```http
GET /api/v1/templates/prescription-templates/:id
Authorization: Bearer <doctor_token>

Response 200:
{
  "success": true,
  "message": "Prescription template retrieved successfully",
  "data": { /* 处方模板详情 */ }
}
```

#### 4. 更新处方模板
```http
PUT /api/v1/templates/prescription-templates/:id
Authorization: Bearer <doctor_token>
Content-Type: application/json

{
  "name": "更新后的模板名称",        // 可选
  "description": "更新后的描述",     // 可选
  "diagnosis": "更新后的诊断",       // 可选
  "medicines": [ /* 药品列表 */ ],   // 可选
  "instructions": "更新后的说明",     // 可选
  "is_active": true                  // 可选
}

Response 200:
{
  "success": true,
  "message": "Prescription template updated successfully",
  "data": { /* 更新后的模板 */ }
}
```

#### 5. 删除处方模板
```http
DELETE /api/v1/templates/prescription-templates/:id
Authorization: Bearer <doctor_token>

Response 200:
{
  "success": true,
  "message": "Prescription template deleted successfully",
  "data": null
}
```

#### 6. 使用处方模板（增加计数）
```http
POST /api/v1/templates/prescription-templates/:id/use
Authorization: Bearer <doctor_token>

Response 200:
{
  "success": true,
  "message": "Usage count updated successfully",
  "data": null
}
```

## 查询参数说明

### 常用语查询参数
- `category`: 分类筛选（diagnosis/advice/symptom）
- `search`: 关键词搜索（搜索内容）
- `is_active`: 状态筛选（true/false）
- `page`: 页码（默认1）
- `page_size`: 每页条数（默认10，最大100）

### 处方模板查询参数
- `search`: 关键词搜索（搜索名称和诊断）
- `is_active`: 状态筛选（true/false）
- `page`: 页码（默认1）
- `page_size`: 每页条数（默认10，最大100）

## 错误响应

### 403 Forbidden
```json
{
  "success": false,
  "message": "Only doctors can access this resource",
  "data": null
}
```

### 404 Not Found
```json
{
  "success": false,
  "message": "Common phrase not found",
  "data": null
}
```

### 400 Bad Request
```json
{
  "success": false,
  "message": "Validation error: content is required",
  "data": null
}
```

## 数据模型

### 常用语分类
- `diagnosis`: 诊断用语
- `advice`: 医嘱建议
- `symptom`: 症状描述

### 药品信息结构
```json
{
  "name": "药品名称",
  "specification": "规格",
  "dosage": "用量",
  "frequency": "用药频率",
  "duration": "用药时长",
  "usage": "用法"
}
```

## 使用场景

1. **快速开方**：医生在开处方时，可以直接选择预定义的处方模板，自动填充所有药品信息
2. **标准化诊断**：使用常用语确保诊断描述的规范性和一致性
3. **提高效率**：减少重复输入，常用的内容会自动排在前面
4. **个性化管理**：每位医生可以根据自己的习惯创建和管理专属模板

## 注意事项

1. **权限限制**：只有角色为`doctor`的用户才能访问这些接口
2. **数据隔离**：医生只能查看和管理自己创建的常用语和模板
3. **使用统计**：系统会自动记录使用次数，并按使用频率排序
4. **软删除**：可以通过`is_active`字段实现软删除，保留历史数据