#!/usr/bin/env python3
"""
Real-Time XAI Trading Platform Startup Script
Launches both backend and frontend services
"""

import subprocess
import sys
import time
import os
import signal
import threading

def run_backend():
    """Run the FastAPI backend server"""
    print("🚀 Starting backend server...")
    subprocess.run([sys.executable, "backend/main.py"])

def run_frontend():
    """Run the Dash frontend dashboard"""
    print("🎨 Starting frontend dashboard...")
    # Wait a bit for backend to start
    time.sleep(3)
    subprocess.run([sys.executable, "frontend/dashboard.py"])

def main():
    print("📈 Real-Time XAI Trading Platform")
    print("=" * 50)
    
    # Check if required directories exist
    if not os.path.exists("backend"):
        print("❌ Backend directory not found. Please ensure the project structure is correct.")
        return
    
    if not os.path.exists("frontend"):
        print("❌ Frontend directory not found. Please ensure the project structure is correct.")
        return
    
    # Start backend in a separate thread
    backend_thread = threading.Thread(target=run_backend, daemon=True)
    backend_thread.start()
    
    # Start frontend
    run_frontend()

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n👋 Shutting down Real-Time XAI Trading Platform...")
        sys.exit(0) 