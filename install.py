#!/usr/bin/env python3
"""
Installation script for Real-Time XAI Trading Platform
"""

import subprocess
import sys
import os
import importlib

def check_python_version():
    """Check if Python version is compatible"""
    if sys.version_info < (3, 8):
        print("❌ Python 3.8 or higher is required")
        return False
    print(f"✅ Python {sys.version_info.major}.{sys.version_info.minor} detected")
    return True

def install_dependencies():
    """Install required dependencies"""
    print("📦 Installing dependencies...")
    try:
        subprocess.check_call([sys.executable, "-m", "pip", "install", "-r", "requirements.txt"])
        print("✅ Dependencies installed successfully")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ Error installing dependencies: {e}")
        return False

def check_dependencies():
    """Check if all required packages are installed"""
    required_packages = [
        "fastapi",
        "uvicorn", 
        "websockets",
        "yfinance",
        "pandas",
        "numpy",
        "plotly",
        "dash",
        "dash-bootstrap-components",
        "scikit-learn",
        "requests",
        "aiohttp"
    ]
    
    missing_packages = []
    
    for package in required_packages:
        try:
            importlib.import_module(package)
            print(f"✅ {package}")
        except ImportError:
            print(f"❌ {package} - Missing")
            missing_packages.append(package)
    
    if missing_packages:
        print(f"\n❌ Missing packages: {', '.join(missing_packages)}")
        return False
    
    print("\n✅ All dependencies are installed")
    return True

def create_directories():
    """Create necessary directories"""
    directories = ["logs", "data", "backtests"]
    
    for directory in directories:
        if not os.path.exists(directory):
            os.makedirs(directory)
            print(f"📁 Created directory: {directory}")
        else:
            print(f"📁 Directory exists: {directory}")

def test_yahoo_finance():
    """Test Yahoo Finance API connection"""
    print("\n🔍 Testing Yahoo Finance API...")
    try:
        import yfinance as yf
        stock = yf.Ticker("AAPL")
        info = stock.info
        if info.get('regularMarketPrice'):
            print(f"✅ Yahoo Finance API working - AAPL price: ${info['regularMarketPrice']:.2f}")
            return True
        else:
            print("❌ Yahoo Finance API not returning data")
            return False
    except Exception as e:
        print(f"❌ Error testing Yahoo Finance API: {e}")
        return False

def main():
    print("🚀 Real-Time XAI Trading Platform Installation")
    print("=" * 50)
    
    # Check Python version
    if not check_python_version():
        return
    
    # Install dependencies
    if not install_dependencies():
        print("\n💡 Try running: pip install -r requirements.txt manually")
        return
    
    # Check dependencies
    if not check_dependencies():
        print("\n💡 Try running: pip install -r requirements.txt manually")
        return
    
    # Create directories
    create_directories()
    
    # Test API connection
    if not test_yahoo_finance():
        print("\n⚠️  Yahoo Finance API test failed. This might be a temporary issue.")
        print("   The platform should still work with cached data.")
    
    print("\n🎉 Installation Complete!")
    print("\nNext Steps:")
    print("1. Start the platform: python run_platform.py")
    print("2. Or start manually:")
    print("   - Backend: python backend/main.py")
    print("   - Frontend: python frontend/dashboard.py")
    print("3. Open http://localhost:8050 in your browser")
    print("4. Run demo: python demo.py")

if __name__ == "__main__":
    main() 