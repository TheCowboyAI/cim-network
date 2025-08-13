#!/usr/bin/env python3
"""
Run All CIM Network Examples

This script runs all available examples to demonstrate the complete
capabilities of the CIM Network SDN system with base topologies.
"""

import asyncio
import sys
from pathlib import Path
import importlib.util

async def run_example(example_path: Path, description: str):
    """Run a single example and handle errors gracefully"""
    print(f"\n{'='*80}")
    print(f"🚀 RUNNING: {description}")
    print(f"📁 File: {example_path.name}")
    print('='*80)
    
    try:
        # Import the example module
        spec = importlib.util.spec_from_file_location("example", example_path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        
        # Run the main function
        if hasattr(module, 'main'):
            await module.main()
        else:
            print("❌ Example does not have a main() function")
            return False
            
        print(f"\n✅ {description} completed successfully!")
        return True
        
    except Exception as e:
        print(f"\n❌ {description} failed: {e}")
        return False

async def main():
    """Run all examples in sequence"""
    print("🌐 CIM Network SDN System - Complete Example Suite")
    print("=" * 65)
    print("This demonstrates the full capabilities of the base topology system:")
    print("• Dev Mode: Single ISP, development environments")
    print("• Leaf Mode: Dual ISP, production environments")  
    print("• Evolution: Growing from dev to production")
    print("• Integration: Complete SDN pipeline testing")
    
    examples_dir = Path(__file__).parent / "examples"
    
    # Define examples to run in order
    examples = [
        ("dev_topology_example.py", "Development Mode Topology Example"),
        ("leaf_topology_example.py", "Leaf Mode Topology Example"), 
        ("topology_progression_example.py", "Topology Evolution Example")
    ]
    
    # Additional tests to include
    additional_tests = [
        ("test_base_topologies.py", "Base Topology Validation Tests"),
        ("test_sdn_pipeline.py", "Complete SDN Pipeline Tests")
    ]
    
    results = []
    
    # Run main examples
    print(f"\n📋 Running {len(examples)} Main Examples:")
    for i, (filename, description) in enumerate(examples, 1):
        example_path = examples_dir / filename
        if example_path.exists():
            print(f"\n[{i}/{len(examples)}] Starting: {description}")
            success = await run_example(example_path, description)
            results.append((description, success))
        else:
            print(f"\n[{i}/{len(examples)}] ❌ Example file not found: {filename}")
            results.append((description, False))
    
    # Run additional validation tests
    print(f"\n📋 Running {len(additional_tests)} Validation Tests:")
    for i, (filename, description) in enumerate(additional_tests, 1):
        test_path = Path(__file__).parent / filename
        if test_path.exists():
            print(f"\n[{i}/{len(additional_tests)}] Starting: {description}")
            
            # For test files, run them directly with Python
            try:
                import subprocess
                result = subprocess.run([
                    sys.executable, str(test_path)
                ], capture_output=True, text=True, cwd=Path(__file__).parent)
                
                if result.returncode == 0:
                    print(f"✅ {description} completed successfully!")
                    print("📄 Test Output Summary:")
                    lines = result.stdout.split('\n')
                    success_lines = [line for line in lines if '✅' in line]
                    for line in success_lines[-5:]:  # Show last 5 success lines
                        print(f"   {line}")
                    results.append((description, True))
                else:
                    print(f"❌ {description} failed!")
                    print("Error output:")
                    print(result.stderr[-500:])  # Show last 500 chars of error
                    results.append((description, False))
                    
            except Exception as e:
                print(f"❌ {description} failed to run: {e}")
                results.append((description, False))
        else:
            print(f"\n[{i}/{len(additional_tests)}] ❌ Test file not found: {filename}")
            results.append((description, False))
    
    # Summary of results
    print(f"\n{'='*80}")
    print("📊 EXECUTION SUMMARY")
    print('='*80)
    
    successful = sum(1 for _, success in results if success)
    total = len(results)
    
    print(f"\n🎯 Overall Results: {successful}/{total} examples completed successfully")
    print(f"📈 Success Rate: {(successful/total*100):.1f}%")
    
    print("\n📋 Detailed Results:")
    for i, (description, success) in enumerate(results, 1):
        status = "✅ PASSED" if success else "❌ FAILED"
        print(f"   {i:2d}. {status} - {description}")
    
    if successful == total:
        print("\n🎉 🎊 ALL EXAMPLES COMPLETED SUCCESSFULLY! 🎊 🎉")
        print("\n🌟 CIM Network SDN Capabilities Demonstrated:")
        print("   ✅ Base topology creation (dev & leaf modes)")
        print("   ✅ Network service integration")
        print("   ✅ nix-topology compliant configuration generation")
        print("   ✅ Production-ready high availability setups")
        print("   ✅ Scalable infrastructure evolution patterns")
        print("   ✅ Complete SDN pipeline validation")
        print("   ✅ Context graph state management")
        print("   ✅ MCP server integration with Claude Code")
        
        print("\n🚀 Next Steps:")
        print("   • Deploy generated NixOS configurations")
        print("   • Integrate with Claude Code MCP settings")
        print("   • Build custom topologies using base templates")
        print("   • Scale from dev to leaf mode as needed")
        
    else:
        failed_examples = [desc for desc, success in results if not success]
        print(f"\n⚠️  {len(failed_examples)} examples failed:")
        for desc in failed_examples:
            print(f"   • {desc}")
        
        print("\n🔧 Troubleshooting:")
        print("   • Ensure Nix development environment is active")
        print("   • Check that all dependencies are installed")
        print("   • Verify MCP server can be imported")
        print("   • Review error messages above for specific issues")
        
        return 1
    
    return 0

if __name__ == "__main__":
    try:
        exit_code = asyncio.run(main())
        sys.exit(exit_code)
    except KeyboardInterrupt:
        print("\n\n⚠️  Execution interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n💥 Fatal error: {e}")
        sys.exit(1)