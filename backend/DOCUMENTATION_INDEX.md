# Documentation Index

This index provides links to all documentation files for the TCM Telemedicine Platform backend.

## Core Documentation

1. **[README.md](README.md)**
   - Project overview
   - Setup instructions
   - API endpoint list
   - Development guidelines

2. **[PROJECT_STATUS.md](PROJECT_STATUS.md)** ✨ NEW
   - Current implementation status
   - Completed features checklist
   - Pending features and roadmap
   - Recommended next steps

3. **[DATABASE_SCHEMA.md](DATABASE_SCHEMA.md)**
   - Complete database schema
   - Table structures and relationships
   - Field descriptions
   - Indexes

4. **[TESTING.md](TESTING.md)**
   - Testing guide
   - How to run tests
   - Writing tests
   - Test structure

5. **[.env.example](.env.example)**
   - Environment variable template
   - Configuration options
   - Required settings

## API Documentation

6. **[API_DOCS_CIRCLE_POST.md](API_DOCS_CIRCLE_POST.md)**
   - Circle post management API
   - Permissions and access control
   - Request/response examples

7. **[API_DOCS_TEMPLATE.md](API_DOCS_TEMPLATE.md)**
   - Template management API
   - Common phrases and prescription templates
   - Doctor-only endpoints

8. **[API_DOCS_PAYMENT.md](API_DOCS_PAYMENT.md)**
   - Payment system API
   - Order management
   - Payment processing
   - Refund handling
   - Balance management

9. **[API_DOCS_VIDEO_CONSULTATION.md](API_DOCS_VIDEO_CONSULTATION.md)**
   - Video consultation API
   - WebRTC signaling
   - Room management
   - Recording features
   - Consultation templates

10. **[API_DOCS_FILE_UPLOAD.md](API_DOCS_FILE_UPLOAD.md)**
    - File upload API
    - Two-step upload process
    - File management
    - Configuration management
    - Storage statistics

## Setup Guides

11. **[PAYMENT_SETUP.md](PAYMENT_SETUP.md)**
   - Payment system configuration
   - WeChat Pay setup
   - Alipay setup
   - Testing payments
   - Troubleshooting

## Development Guides

12. **[../CLAUDE.md](../CLAUDE.md)**
   - Comprehensive development guide
   - System architecture
   - Business requirements
   - Implementation roadmap
   - Chinese/English bilingual

## Quick Links

### For New Developers
1. Start with [README.md](README.md)
2. Check current status in [PROJECT_STATUS.md](PROJECT_STATUS.md)
3. Set up environment using [.env.example](.env.example)
4. Understand database with [DATABASE_SCHEMA.md](DATABASE_SCHEMA.md)
5. Run tests following [TESTING.md](TESTING.md)

### For API Integration
- Authentication: See [README.md](README.md#authentication)
- Payment APIs: See [API_DOCS_PAYMENT.md](API_DOCS_PAYMENT.md)
- Video Consultation APIs: See [API_DOCS_VIDEO_CONSULTATION.md](API_DOCS_VIDEO_CONSULTATION.md)
- File Upload APIs: See [API_DOCS_FILE_UPLOAD.md](API_DOCS_FILE_UPLOAD.md)
- Community APIs: See [API_DOCS_CIRCLE_POST.md](API_DOCS_CIRCLE_POST.md)
- Template APIs: See [API_DOCS_TEMPLATE.md](API_DOCS_TEMPLATE.md)

### For System Administrators
- Payment setup: See [PAYMENT_SETUP.md](PAYMENT_SETUP.md)
- Database schema: See [DATABASE_SCHEMA.md](DATABASE_SCHEMA.md)
- Environment config: See [.env.example](.env.example)

## Documentation Status

### Up to Date ✅
- README.md (Updated with project status link)
- PROJECT_STATUS.md (New - comprehensive status overview)
- DATABASE_SCHEMA.md (Includes all tables)
- API_DOCS_PAYMENT.md
- API_DOCS_VIDEO_CONSULTATION.md
- API_DOCS_FILE_UPLOAD.md
- PAYMENT_SETUP.md
- TESTING.md
- .env.example

### May Need Updates 🔄
- API documentation for other modules (notification, statistics, review)
- Deployment guide
- Performance tuning guide
- WebRTC/Video setup guide
- OSS/S3 integration guide
- Redis integration guide

## Contributing to Documentation

When adding new features:
1. Update relevant API docs
2. Add to DATABASE_SCHEMA.md if new tables
3. Update README.md endpoint list
4. Add setup instructions if needed
5. Update this index

---

Last updated: 2024-01-21