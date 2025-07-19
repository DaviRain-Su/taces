# Video Consultation API Documentation

## Overview
The Video Consultation module provides real-time video communication between doctors and patients using WebRTC technology. It includes features for consultation management, WebRTC signaling, recording, and templates.

## Table of Contents
- [Authentication](#authentication)
- [Consultation Management](#consultation-management)
- [Room Management](#room-management)
- [WebRTC Signaling](#webrtc-signaling)
- [Recording Management](#recording-management)
- [Template Management](#template-management)
- [Statistics](#statistics)
- [Data Models](#data-models)
- [Error Codes](#error-codes)
- [WebRTC Integration Guide](#webrtc-integration-guide)

## Authentication
All endpoints require JWT authentication. Include the token in the Authorization header:
```
Authorization: Bearer <jwt_token>
```

## Consultation Management

### Create Video Consultation
Creates a new video consultation session.

**Endpoint:** `POST /api/v1/video-consultations`

**Access:** Doctor, Admin

**Request Body:**
```json
{
  "appointment_id": "uuid",
  "doctor_id": "uuid",
  "patient_id": "uuid",
  "scheduled_start_time": "2024-01-20T10:00:00Z",
  "chief_complaint": "头痛、失眠"
}
```

**Response:**
```json
{
  "success": true,
  "message": "视频问诊创建成功",
  "data": {
    "id": "uuid",
    "appointment_id": "uuid",
    "doctor_id": "uuid",
    "patient_id": "uuid",
    "room_id": "room_abc123def456",
    "status": "waiting",
    "scheduled_start_time": "2024-01-20T10:00:00Z",
    "chief_complaint": "头痛、失眠",
    "created_at": "2024-01-19T08:00:00Z"
  }
}
```

### Get Consultation Details
Retrieves details of a specific consultation.

**Endpoint:** `GET /api/v1/video-consultations/:id`

**Access:** Doctor (own), Patient (own), Admin

**Response:**
```json
{
  "success": true,
  "message": "获取视频问诊成功",
  "data": {
    "id": "uuid",
    "appointment_id": "uuid",
    "doctor_id": "uuid",
    "patient_id": "uuid",
    "room_id": "room_abc123def456",
    "status": "completed",
    "scheduled_start_time": "2024-01-20T10:00:00Z",
    "actual_start_time": "2024-01-20T10:02:00Z",
    "end_time": "2024-01-20T10:30:00Z",
    "duration": 1680,
    "chief_complaint": "头痛、失眠",
    "diagnosis": "神经性头痛，失眠症",
    "treatment_plan": "中药调理，针灸治疗",
    "notes": "患者症状明显，建议坚持治疗",
    "connection_quality": "good",
    "patient_rating": 5,
    "patient_feedback": "医生很专业，解答详细"
  }
}
```

### List Consultations
Lists consultations with filtering options.

**Endpoint:** `GET /api/v1/video-consultations`

**Access:** All authenticated users (filtered by role)

**Query Parameters:**
- `doctor_id` (optional): Filter by doctor
- `patient_id` (optional): Filter by patient
- `status` (optional): Filter by status
- `date_from` (optional): Start date filter
- `date_to` (optional): End date filter
- `page` (optional): Page number (default: 1)
- `page_size` (optional): Items per page (default: 20, max: 100)

**Response:**
```json
{
  "success": true,
  "message": "获取视频问诊列表成功",
  "data": [
    {
      "id": "uuid",
      "room_id": "room_abc123def456",
      "status": "waiting",
      "scheduled_start_time": "2024-01-20T10:00:00Z",
      // ... other fields
    }
  ]
}
```

### Start Consultation
Starts a video consultation session.

**Endpoint:** `PUT /api/v1/video-consultations/:id/start`

**Access:** Doctor only

**Response:**
```json
{
  "success": true,
  "message": "问诊已开始",
  "data": {}
}
```

### End Consultation
Ends a video consultation session with completion details.

**Endpoint:** `PUT /api/v1/video-consultations/:id/end`

**Access:** Doctor only

**Request Body:**
```json
{
  "diagnosis": "神经性头痛，失眠症",
  "treatment_plan": "中药调理，针灸治疗",
  "notes": "患者症状明显，建议坚持治疗"
}
```

**Response:**
```json
{
  "success": true,
  "message": "问诊已结束",
  "data": {}
}
```

### Update Consultation
Updates consultation information.

**Endpoint:** `PUT /api/v1/video-consultations/:id`

**Access:** Doctor only

**Request Body:**
```json
{
  "chief_complaint": "头痛、失眠、焦虑",
  "diagnosis": "神经性头痛，失眠症，轻度焦虑",
  "treatment_plan": "中药调理，针灸治疗，心理疏导",
  "notes": "增加焦虑症状的治疗"
}
```

### Rate Consultation
Allows patients to rate and provide feedback for a consultation.

**Endpoint:** `POST /api/v1/video-consultations/:id/rate`

**Access:** Patient only

**Request Body:**
```json
{
  "rating": 5,
  "feedback": "医生很专业，解答详细"
}
```

## Room Management

### Join Room
Joins a video consultation room and receives connection credentials.

**Endpoint:** `POST /api/v1/video-consultations/room/:room_id/join`

**Access:** Doctor or Patient (must be participant)

**Response:**
```json
{
  "success": true,
  "message": "加入房间成功",
  "data": {
    "room_id": "room_abc123def456",
    "token": "eyJhbGciOiJIUzI1NiIs...",
    "ice_servers": [
      {
        "urls": ["stun:stun.l.google.com:19302"]
      }
    ],
    "role": "doctor"
  }
}
```

## WebRTC Signaling

### Send Signal
Sends a WebRTC signaling message to another participant.

**Endpoint:** `POST /api/v1/video-consultations/signal`

**Access:** Room participants only

**Request Body:**
```json
{
  "room_id": "room_abc123def456",
  "to_user_id": "uuid",
  "signal_type": "offer",
  "payload": {
    "sdp": "v=0\r\no=- 123456789..."
  }
}
```

**Signal Types:**
- `offer`: WebRTC offer
- `answer`: WebRTC answer
- `ice_candidate`: ICE candidate
- `join`: Join notification
- `leave`: Leave notification
- `error`: Error message

### Receive Signals
Retrieves pending WebRTC signals for the current user.

**Endpoint:** `GET /api/v1/video-consultations/signal/:room_id`

**Access:** Room participants only

**Response:**
```json
{
  "success": true,
  "message": "获取信令成功",
  "data": [
    {
      "id": "uuid",
      "room_id": "room_abc123def456",
      "from_user_id": "uuid",
      "to_user_id": "uuid",
      "signal_type": "offer",
      "payload": {
        "sdp": "v=0\r\no=- 123456789..."
      },
      "created_at": "2024-01-20T10:00:00Z"
    }
  ]
}
```

## Recording Management

### Start Recording
Starts recording a video consultation.

**Endpoint:** `POST /api/v1/video-consultations/:id/recording/start`

**Access:** Doctor only

**Response:**
```json
{
  "success": true,
  "message": "录制已开始",
  "data": {
    "id": "uuid",
    "consultation_id": "uuid",
    "status": "recording",
    "started_at": "2024-01-20T10:05:00Z"
  }
}
```

### Complete Recording
Marks a recording as completed (called by recording service).

**Endpoint:** `PUT /api/v1/video-consultations/recording/:id/complete`

**Access:** Admin only

**Request Body:**
```json
{
  "recording_url": "https://cdn.example.com/recordings/abc123.mp4",
  "file_size": 104857600,
  "duration": 1800
}
```

### Get Recording Details
Retrieves details of a specific recording.

**Endpoint:** `GET /api/v1/video-consultations/recording/:id`

**Access:** Consultation participants, Admin

**Response:**
```json
{
  "success": true,
  "message": "获取录制记录成功",
  "data": {
    "id": "uuid",
    "consultation_id": "uuid",
    "recording_url": "https://cdn.example.com/recordings/abc123.mp4",
    "thumbnail_url": "https://cdn.example.com/thumbnails/abc123.jpg",
    "file_size": 104857600,
    "duration": 1800,
    "format": "mp4",
    "status": "completed",
    "started_at": "2024-01-20T10:05:00Z",
    "completed_at": "2024-01-20T10:35:00Z"
  }
}
```

### List Consultation Recordings
Lists all recordings for a consultation.

**Endpoint:** `GET /api/v1/video-consultations/:id/recordings`

**Access:** Consultation participants, Admin

## Template Management

### Create Consultation Template
Creates a new consultation template for quick reuse.

**Endpoint:** `POST /api/v1/video-consultations/templates`

**Access:** Doctor only

**Request Body:**
```json
{
  "name": "失眠症常规诊断",
  "chief_complaint": "失眠、多梦、早醒",
  "diagnosis": "失眠症",
  "treatment_plan": "安神补脑液，酸枣仁汤加减",
  "notes": "注意作息规律，避免熬夜"
}
```

### List Doctor Templates
Lists all templates for the authenticated doctor.

**Endpoint:** `GET /api/v1/video-consultations/templates`

**Access:** Doctor only

**Response:**
```json
{
  "success": true,
  "message": "获取模板列表成功",
  "data": [
    {
      "id": "uuid",
      "name": "失眠症常规诊断",
      "chief_complaint": "失眠、多梦、早醒",
      "diagnosis": "失眠症",
      "treatment_plan": "安神补脑液，酸枣仁汤加减",
      "notes": "注意作息规律，避免熬夜",
      "usage_count": 15,
      "created_at": "2024-01-01T08:00:00Z"
    }
  ]
}
```

### Use Template
Increments the usage count of a template.

**Endpoint:** `POST /api/v1/video-consultations/templates/:id/use`

**Access:** Doctor (owner only)

## Statistics

### Get Consultation Statistics
Retrieves video consultation statistics.

**Endpoint:** `GET /api/v1/video-consultations/statistics`

**Access:** Doctor (own stats), Admin (all stats)

**Query Parameters:**
- `doctor_id` (optional): Filter by doctor (Admin only)
- `start_date` (optional): Start date for statistics
- `end_date` (optional): End date for statistics

**Response:**
```json
{
  "success": true,
  "message": "获取统计数据成功",
  "data": {
    "total_consultations": 150,
    "completed_consultations": 140,
    "average_duration": 1680.5,
    "average_rating": 4.8,
    "no_show_rate": 2.5
  }
}
```

## Data Models

### ConsultationStatus
- `waiting`: Waiting to start
- `in_progress`: Currently in session
- `completed`: Successfully completed
- `cancelled`: Cancelled by user
- `no_show`: Patient didn't show up

### ConnectionQuality
- `excellent`: Excellent connection
- `good`: Good connection
- `fair`: Fair connection
- `poor`: Poor connection

### SignalType
- `offer`: WebRTC offer
- `answer`: WebRTC answer
- `ice_candidate`: ICE candidate
- `join`: Join room
- `leave`: Leave room
- `error`: Error signal

## Error Codes
- `400`: Bad Request - Invalid parameters
- `401`: Unauthorized - Missing or invalid token
- `403`: Forbidden - Insufficient permissions
- `404`: Not Found - Resource not found
- `409`: Conflict - Resource conflict (e.g., consultation already started)
- `500`: Internal Server Error

## WebRTC Integration Guide

### Client-Side Implementation

1. **Join Room**
   ```javascript
   const response = await fetch(`/api/v1/video-consultations/room/${roomId}/join`, {
     method: 'POST',
     headers: {
       'Authorization': `Bearer ${token}`,
       'Content-Type': 'application/json'
     }
   });
   const { data } = await response.json();
   ```

2. **Initialize WebRTC**
   ```javascript
   const pc = new RTCPeerConnection({
     iceServers: data.ice_servers
   });
   ```

3. **Send Signals**
   ```javascript
   // Send offer
   const offer = await pc.createOffer();
   await pc.setLocalDescription(offer);
   
   await fetch('/api/v1/video-consultations/signal', {
     method: 'POST',
     headers: {
       'Authorization': `Bearer ${token}`,
       'Content-Type': 'application/json'
     },
     body: JSON.stringify({
       room_id: roomId,
       to_user_id: remoteUserId,
       signal_type: 'offer',
       payload: { sdp: offer.sdp }
     })
   });
   ```

4. **Receive Signals (Polling)**
   ```javascript
   const pollSignals = async () => {
     const response = await fetch(`/api/v1/video-consultations/signal/${roomId}`, {
       headers: {
         'Authorization': `Bearer ${token}`
       }
     });
     const { data: signals } = await response.json();
     
     for (const signal of signals) {
       await handleSignal(signal);
     }
     
     setTimeout(pollSignals, 1000); // Poll every second
   };
   ```

### Best Practices
1. Use STUN/TURN servers for NAT traversal
2. Implement connection quality monitoring
3. Handle reconnection logic
4. Record important events (join, leave, errors)
5. Implement proper error handling and user feedback
6. Use secure WebSocket for real-time signaling (future enhancement)

### Security Considerations
1. Tokens are room-specific and time-limited
2. Only consultation participants can join the room
3. Signals are authenticated and validated
4. Recording URLs are signed and time-limited
5. All communications should use HTTPS/WSS