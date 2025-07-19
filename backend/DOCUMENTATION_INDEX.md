# Documentation Index

This index provides links to all documentation files for the TCM Telemedicine Platform backend.

## Core Documentation

1. **[README.md](README.md)**
   - Project overview
   - Setup instructions
   - API endpoint list
   - Development guidelines

2. **[DATABASE_SCHEMA.md](DATABASE_SCHEMA.md)**
   - Complete database schema
   - Table structures and relationships
   - Field descriptions
   - Indexes

3. **[TESTING.md](TESTING.md)**
   - Testing guide
   - How to run tests
   - Writing tests
   - Test structure

4. **[.env.example](.env.example)**
   - Environment variable template
   - Configuration options
   - Required settings

## API Documentation

5. **[API_DOCS_CIRCLE_POST.md](API_DOCS_CIRCLE_POST.md)**
   - Circle post management API
   - Permissions and access control
   - Request/response examples

6. **[API_DOCS_TEMPLATE.md](API_DOCS_TEMPLATE.md)**
   - Template management API
   - Common phrases and prescription templates
   - Doctor-only endpoints

7. **[API_DOCS_PAYMENT.md](API_DOCS_PAYMENT.md)**
   - Payment system API
   - Order management
   - Payment processing
   - Refund handling
   - Balance management

8. **[API_DOCS_VIDEO_CONSULTATION.md](API_DOCS_VIDEO_CONSULTATION.md)** ✨ NEW
   - Video consultation API
   - WebRTC signaling
   - Room management
   - Recording features
   - Consultation templates

9. **[API_DOCS_FILE_UPLOAD.md](API_DOCS_FILE_UPLOAD.md)** ✨ NEW
   - File upload API
   - Two-step upload process
   - File management
   - Configuration management
   - Storage statistics

## Setup Guides

10. **[PAYMENT_SETUP.md](PAYMENT_SETUP.md)**
   - Payment system configuration
   - WeChat Pay setup
   - Alipay setup
   - Testing payments
   - Troubleshooting

## Development Guides

11. **[../CLAUDE.md](../CLAUDE.md)**
   - Comprehensive development guide
   - System architecture
   - Business requirements
   - Implementation roadmap
   - Chinese/English bilingual

## Quick Links

### For New Developers
1. Start with [README.md](README.md)
2. Set up environment using [.env.example](.env.example)
3. Understand database with [DATABASE_SCHEMA.md](DATABASE_SCHEMA.md)
4. Run tests following [TESTING.md](TESTING.md)

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
- README.md (Updated with video consultation and file upload)
- DATABASE_SCHEMA.md (Includes all tables including video consultation and file upload)
- API_DOCS_PAYMENT.md
- API_DOCS_VIDEO_CONSULTATION.md (New)
- API_DOCS_FILE_UPLOAD.md (New)
- PAYMENT_SETUP.md
- TESTING.md
- .env.example

### May Need Updates 🔄
- API documentation for other modules (notification, statistics, review)
- Deployment guide
- Performance tuning guide
- WebRTC/Video setup guide
- OSS/S3 integration guide

## Contributing to Documentation

When adding new features:
1. Update relevant API docs
2. Add to DATABASE_SCHEMA.md if new tables
3. Update README.md endpoint list
4. Add setup instructions if needed
5. Update this index

---

Last updated: 2024-01-21