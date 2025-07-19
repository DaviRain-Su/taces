# 圈子帖子管理 API 文档

## 概述
圈子帖子功能允许圈子成员发布帖子、点赞和评论，实现社区互动。

## 权限说明
- **发帖**：仅圈子成员可发帖
- **编辑/删除帖子**：仅作者可编辑，作者和管理员可删除
- **点赞**：所有已登录用户
- **评论**：所有已登录用户
- **删除评论**：仅评论作者和管理员

## API 端点

### 1. 创建帖子
```http
POST /api/v1/posts
Authorization: Bearer <token>
Content-Type: application/json

{
  "circle_id": "00000000-0000-0000-0000-000000000001",
  "title": "帖子标题",
  "content": "帖子内容",
  "images": ["image1.jpg", "image2.jpg"]  // 可选，最多9张
}

Response 200:
{
  "success": true,
  "message": "Post created successfully",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "author_id": "user-uuid",
    "circle_id": "circle-uuid",
    "title": "帖子标题",
    "content": "帖子内容",
    "images": ["image1.jpg", "image2.jpg"],
    "likes": 0,
    "comments": 0,
    "status": "active",
    "created_at": "2024-01-04T12:00:00Z",
    "updated_at": "2024-01-04T12:00:00Z"
  }
}
```

### 2. 获取帖子列表
```http
GET /api/v1/posts?circle_id=xxx&author_id=xxx&page=1&page_size=10
Authorization: Bearer <token>

Response 200:
{
  "success": true,
  "message": "Posts retrieved successfully",
  "data": {
    "posts": [
      {
        "id": "post-uuid",
        "author_id": "user-uuid",
        "author_name": "用户名称",
        "circle_id": "circle-uuid",
        "circle_name": "圈子名称",
        "title": "帖子标题",
        "content": "帖子内容",
        "images": ["image1.jpg"],
        "likes": 5,
        "comments": 3,
        "is_liked": true,  // 当前用户是否点赞
        "created_at": "2024-01-04T12:00:00Z",
        "updated_at": "2024-01-04T12:00:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "page_size": 10,
      "total": 100,
      "total_pages": 10
    }
  }
}
```

### 3. 获取帖子详情
```http
GET /api/v1/posts/:id
Authorization: Bearer <token>

Response 200:
{
  "success": true,
  "message": "Post retrieved successfully",
  "data": {
    "id": "post-uuid",
    "author_id": "user-uuid",
    "author_name": "用户名称",
    "circle_id": "circle-uuid",
    "circle_name": "圈子名称",
    "title": "帖子标题",
    "content": "帖子内容",
    "images": ["image1.jpg", "image2.jpg"],
    "likes": 5,
    "comments": 3,
    "is_liked": false,
    "created_at": "2024-01-04T12:00:00Z",
    "updated_at": "2024-01-04T12:00:00Z"
  }
}
```

### 4. 更新帖子
```http
PUT /api/v1/posts/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "更新后的标题",    // 可选
  "content": "更新后的内容",  // 可选
  "images": ["new1.jpg"]      // 可选
}

Response 200:
{
  "success": true,
  "message": "Post updated successfully",
  "data": { /* 更新后的帖子信息 */ }
}
```

### 5. 删除帖子
```http
DELETE /api/v1/posts/:id
Authorization: Bearer <token>

Response 200:
{
  "success": true,
  "message": "Post deleted successfully",
  "data": null
}
```

### 6. 点赞/取消点赞
```http
POST /api/v1/posts/:id/like
Authorization: Bearer <token>

Response 200:
{
  "success": true,
  "message": "Post liked successfully",  // 或 "Post unliked successfully"
  "data": {
    "liked": true  // true=点赞, false=取消点赞
  }
}
```

### 7. 获取评论列表
```http
GET /api/v1/posts/:id/comments?page=1&page_size=20
Authorization: Bearer <token>

Response 200:
{
  "success": true,
  "message": "Comments retrieved successfully",
  "data": {
    "comments": [
      {
        "id": "comment-uuid",
        "post_id": "post-uuid",
        "user_id": "user-uuid",
        "user_name": "评论者名称",
        "content": "评论内容",
        "is_deleted": false,
        "created_at": "2024-01-04T12:00:00Z",
        "updated_at": "2024-01-04T12:00:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "page_size": 20,
      "total": 50,
      "total_pages": 3
    }
  }
}
```

### 8. 发表评论
```http
POST /api/v1/posts/:id/comments
Authorization: Bearer <token>
Content-Type: application/json

{
  "content": "评论内容"
}

Response 200:
{
  "success": true,
  "message": "Comment created successfully",
  "data": {
    "id": "comment-uuid",
    "post_id": "post-uuid",
    "user_id": "user-uuid",
    "content": "评论内容",
    "is_deleted": false,
    "created_at": "2024-01-04T12:00:00Z",
    "updated_at": "2024-01-04T12:00:00Z"
  }
}
```

### 9. 删除评论
```http
DELETE /api/v1/comments/:id
Authorization: Bearer <token>

Response 200:
{
  "success": true,
  "message": "Comment deleted successfully",
  "data": null
}
```

### 10. 获取用户的帖子
```http
GET /api/v1/users/:user_id/posts?page=1&page_size=10
Authorization: Bearer <token>

Response: 同帖子列表接口
```

### 11. 获取圈子的帖子
```http
GET /api/v1/circles/:circle_id/posts?page=1&page_size=10
Authorization: Bearer <token>

Response: 同帖子列表接口
```

## 错误响应

### 400 Bad Request
```json
{
  "success": false,
  "message": "Content contains sensitive words",
  "data": null
}
```

### 403 Forbidden
```json
{
  "success": false,
  "message": "You must be a member of the circle to post",
  "data": null
}
```

### 404 Not Found
```json
{
  "success": false,
  "message": "Post not found",
  "data": null
}
```

## 数据验证规则

### 创建帖子
- `title`: 必填，1-200字符
- `content`: 必填，1-5000字符
- `images`: 可选，数组，最多9张图片
- `circle_id`: 必填，有效的圈子ID

### 创建评论
- `content`: 必填，1-1000字符

## 业务规则

1. **发帖权限**：只有圈子成员才能在该圈子发帖
2. **敏感词过滤**：帖子标题、内容和评论都会进行敏感词检查
3. **软删除**：帖子和评论都采用软删除，保留数据但不显示
4. **计数器更新**：
   - 发帖时，圈子的帖子数+1
   - 删除帖子时，圈子的帖子数-1
   - 点赞/取消点赞时，帖子的点赞数相应增减
   - 评论/删除评论时，帖子的评论数相应增减

## 性能优化

1. **分页查询**：所有列表接口都支持分页，默认每页10条
2. **索引优化**：在 `circle_id`、`author_id`、`created_at` 等字段建立索引
3. **缓存策略**：可以对热门帖子和评论进行缓存
4. **并发控制**：点赞和评论计数使用事务保证数据一致性