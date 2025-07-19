# TCM Telemedicine Platform Backend

## Overview
This is the backend service for the TCM Telemedicine Platform, built with Rust using the Axum web framework.

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

### Circle Management
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