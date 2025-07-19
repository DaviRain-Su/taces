# TCM Telemedicine Platform Backend

## Overview
This is the backend service for the TCM Telemedicine Platform (香河香草中医诊所多端诊疗平台), built with Rust using the Axum web framework. The platform provides comprehensive traditional Chinese medicine telemedicine services including appointment management, prescription handling, doctor-patient interactions, content management, and more.

### Key Features
- **User Management**: Multi-role support (Admin, Doctor, Patient)
- **Appointment System**: Online appointment booking with time slot management
- **Prescription Management**: Digital prescription creation and tracking
- **Content Platform**: Articles and videos for health education
- **Live Streaming**: Doctor-hosted health education streams
- **Community Features**: Patient circles and discussion forums
- **Notification System**: Multi-channel notifications (In-app, SMS, Email, Push)
- **Analytics Dashboard**: Comprehensive statistics and data export
- **Review System**: Patient feedback and ratings
- **Template Management**: Reusable prescription and phrase templates

## Setup

### Prerequisites
- Rust 1.70+
- MySQL 8.0+
- `.env` file with required environment variables

### Environment Variables
Copy `.env.example` to `.env` and configure:
```bash
cp .env.example .env
```

### Database Setup
1. Create a MySQL database
2. Update `DATABASE_URL` in `.env`
3. Run migrations (automatic on startup)

### Running the Server
```bash
cargo run
```

The server will start on `http://localhost:3000`

## API Endpoints

### Authentication
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - Login user

### User Management
- `GET /api/v1/users` - List users (Admin only)
- `GET /api/v1/users/:id` - Get user by ID
- `PUT /api/v1/users/:id` - Update user
- `DELETE /api/v1/users/:id` - Delete user (Admin only)
- `DELETE /api/v1/users/batch/delete` - Batch delete users (Admin only)
- `GET /api/v1/users/batch/export` - Export users as CSV (Admin only)

### Doctor Management
- `GET /api/v1/doctors` - List doctors
- `GET /api/v1/doctors/:id` - Get doctor by ID
- `POST /api/v1/doctors` - Create doctor profile (Admin only)
- `PUT /api/v1/doctors/:id` - Update doctor
- `PUT /api/v1/doctors/:id/photos` - Update doctor photos
- `GET /api/v1/doctors/by-user/:user_id` - Get doctor by user ID

### Appointment Management
- `GET /api/v1/appointments` - List appointments
- `GET /api/v1/appointments/:id` - Get appointment by ID
- `POST /api/v1/appointments` - Create appointment
- `PUT /api/v1/appointments/:id` - Update appointment
- `PUT /api/v1/appointments/:id/cancel` - Cancel appointment
- `GET /api/v1/appointments/doctor/:doctor_id` - Get doctor's appointments
- `GET /api/v1/appointments/patient/:patient_id` - Get patient's appointments
- `GET /api/v1/appointments/available-slots` - Get available time slots

### Prescription Management
- `GET /api/v1/prescriptions` - List prescriptions (Admin only)
- `GET /api/v1/prescriptions/:id` - Get prescription by ID
- `POST /api/v1/prescriptions` - Create prescription (Doctor only)
- `GET /api/v1/prescriptions/code/:code` - Get prescription by code
- `GET /api/v1/prescriptions/doctor/:doctor_id` - Get doctor's prescriptions
- `GET /api/v1/prescriptions/patient/:patient_id` - Get patient's prescriptions

### Department Management
- `GET /api/v1/departments` - List departments
- `GET /api/v1/departments/:id` - Get department by ID
- `POST /api/v1/departments` - Create department (Admin only)
- `PUT /api/v1/departments/:id` - Update department (Admin only)
- `DELETE /api/v1/departments/:id` - Delete department (Admin only)

### Patient Group Management
- `GET /api/v1/patient-groups` - List doctor's patient groups
- `GET /api/v1/patient-groups/:id` - Get patient group by ID
- `POST /api/v1/patient-groups` - Create patient group (Doctor only)
- `PUT /api/v1/patient-groups/:id` - Update patient group
- `DELETE /api/v1/patient-groups/:id` - Delete patient group
- `POST /api/v1/patient-groups/:id/members` - Add patients to group
- `DELETE /api/v1/patient-groups/:id/members/:patient_id` - Remove patient from group
- `POST /api/v1/patient-groups/:id/send-message` - Send message to group

### Patient Profile Management
- `GET /api/v1/patient-profiles` - List user's patient profiles
- `GET /api/v1/patient-profiles/:id` - Get patient profile by ID
- `POST /api/v1/patient-profiles` - Create patient profile
- `PUT /api/v1/patient-profiles/:id` - Update patient profile
- `DELETE /api/v1/patient-profiles/:id` - Delete patient profile
- `PUT /api/v1/patient-profiles/:id/set-default` - Set as default profile

### Content Management
- `GET /api/v1/content/articles` - List articles
- `GET /api/v1/content/articles/:id` - Get article by ID
- `POST /api/v1/content/articles` - Create article (Doctor/Admin only)
- `PUT /api/v1/content/articles/:id` - Update article
- `DELETE /api/v1/content/articles/:id` - Delete article
- `PUT /api/v1/content/articles/:id/view` - Increment view count
- `GET /api/v1/content/videos` - List videos
- `GET /api/v1/content/videos/:id` - Get video by ID
- `POST /api/v1/content/videos` - Create video (Doctor/Admin only)
- `PUT /api/v1/content/videos/:id` - Update video
- `DELETE /api/v1/content/videos/:id` - Delete video
- `PUT /api/v1/content/videos/:id/view` - Increment view count
- `GET /api/v1/content/categories` - List categories
- `POST /api/v1/content/categories` - Create category (Admin only)
- `PUT /api/v1/content/categories/:id` - Update category (Admin only)
- `DELETE /api/v1/content/categories/:id` - Delete category (Admin only)

### Live Stream Management
- `GET /api/v1/live-streams` - List live streams
- `GET /api/v1/live-streams/:id` - Get live stream by ID
- `POST /api/v1/live-streams` - Create live stream (Doctor only)
- `PUT /api/v1/live-streams/:id` - Update live stream
- `DELETE /api/v1/live-streams/:id` - Delete live stream
- `PUT /api/v1/live-streams/:id/start` - Start live stream
- `PUT /api/v1/live-streams/:id/end` - End live stream
- `GET /api/v1/live-streams/upcoming` - Get upcoming live streams
- `GET /api/v1/live-streams/my` - Get my live streams (Doctor)

### Circle (Community) Management
- `POST /api/v1/circles` - Create circle
- `GET /api/v1/circles` - List circles (with search/filter)
- `GET /api/v1/circles/:id` - Get circle details
- `PUT /api/v1/circles/:id` - Update circle
- `DELETE /api/v1/circles/:id` - Delete circle (soft delete)
- `POST /api/v1/circles/:id/join` - Join circle
- `POST /api/v1/circles/:id/leave` - Leave circle
- `GET /api/v1/circles/:id/members` - Get circle members
- `PUT /api/v1/circles/:id/members/:user_id/role` - Update member role
- `DELETE /api/v1/circles/:id/members/:user_id` - Remove member
- `GET /api/v1/my-circles` - Get user's joined circles

### Circle Post Management
- `GET /api/v1/posts` - List posts (with filters)
- `GET /api/v1/posts/:id` - Get post by ID
- `POST /api/v1/posts` - Create post (Circle members only)
- `PUT /api/v1/posts/:id` - Update post (Author only)
- `DELETE /api/v1/posts/:id` - Delete post (Author/Admin only)
- `GET /api/v1/users/:user_id/posts` - Get user's posts
- `GET /api/v1/circles/:circle_id/posts` - Get circle's posts
- `POST /api/v1/posts/:id/like` - Toggle like on post
- `GET /api/v1/posts/:id/comments` - Get post comments
- `POST /api/v1/posts/:id/comments` - Create comment
- `DELETE /api/v1/comments/:id` - Delete comment (Author/Admin only)

### Template Management (Doctor only)
- `GET /api/v1/templates/common-phrases` - List common phrases
- `GET /api/v1/templates/common-phrases/:id` - Get common phrase by ID
- `POST /api/v1/templates/common-phrases` - Create common phrase
- `PUT /api/v1/templates/common-phrases/:id` - Update common phrase
- `DELETE /api/v1/templates/common-phrases/:id` - Delete common phrase
- `POST /api/v1/templates/common-phrases/:id/use` - Increment usage count
- `GET /api/v1/templates/prescription-templates` - List prescription templates
- `GET /api/v1/templates/prescription-templates/:id` - Get prescription template by ID
- `POST /api/v1/templates/prescription-templates` - Create prescription template
- `PUT /api/v1/templates/prescription-templates/:id` - Update prescription template
- `DELETE /api/v1/templates/prescription-templates/:id` - Delete prescription template
- `POST /api/v1/templates/prescription-templates/:id/use` - Increment usage count

### Review Management
- `GET /api/v1/reviews` - List reviews (Admin only)
- `GET /api/v1/reviews/:id` - Get review by ID
- `POST /api/v1/reviews` - Create review (Patient only, after completed appointment)
- `PUT /api/v1/reviews/:id` - Update review (Author only)
- `POST /api/v1/reviews/:id/reply` - Reply to review (Doctor only)
- `PUT /api/v1/reviews/:id/visibility` - Update review visibility (Admin only)
- `GET /api/v1/reviews/doctor/:doctor_id/reviews` - Get doctor's reviews (Public)
- `GET /api/v1/reviews/doctor/:doctor_id/statistics` - Get doctor's review statistics (Public)
- `GET /api/v1/reviews/patient/:patient_id/reviews` - Get patient's reviews
- `GET /api/v1/reviews/tags` - Get review tags (Public)
- `POST /api/v1/reviews/tags` - Create review tag (Admin only)

### Notification System
- `GET /api/v1/notifications` - Get user notifications (with pagination and filters)
- `GET /api/v1/notifications/:id` - Get notification details
- `PUT /api/v1/notifications/:id/read` - Mark notification as read
- `PUT /api/v1/notifications/read-all` - Mark all notifications as read
- `DELETE /api/v1/notifications/:id` - Delete notification (soft delete)
- `GET /api/v1/notifications/stats` - Get notification statistics
- `GET /api/v1/notifications/settings` - Get notification settings
- `PUT /api/v1/notifications/settings` - Update notification settings
- `POST /api/v1/notifications/push-token` - Register push notification token
- `POST /api/v1/notifications/announcement` - Send system announcement (Admin only)

### Statistics and Analytics
#### Public Statistics
- `GET /api/v1/statistics/departments` - Department statistics
- `GET /api/v1/statistics/top-doctors` - Top 10 doctors by appointments
- `GET /api/v1/statistics/top-content` - Top 10 popular content

#### Protected Statistics
- `GET /api/v1/statistics/dashboard` - Admin dashboard statistics (Admin only)
- `GET /api/v1/statistics/doctor/:doctor_id` - Doctor performance statistics
- `GET /api/v1/statistics/patient` - Patient activity statistics
- `GET /api/v1/statistics/appointment-trends` - Appointment trends over time (Admin only)
- `GET /api/v1/statistics/time-slots` - Time slot distribution (Admin only)
- `GET /api/v1/statistics/content` - Content statistics (Admin only)
- `GET /api/v1/statistics/live-streams` - Live stream statistics (Admin only)
- `GET /api/v1/statistics/circles` - Circle/community statistics (Admin only)
- `GET /api/v1/statistics/user-growth` - User growth trends (Admin only)
- `GET /api/v1/statistics/appointment-heatmap` - Appointment heatmap by hour/day (Admin only)
- `GET /api/v1/statistics/export` - Export data to CSV/Excel (Admin only)

## Authentication
All endpoints except authentication endpoints require a Bearer token in the Authorization header:
```
Authorization: Bearer <token>
```

## Development
```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```