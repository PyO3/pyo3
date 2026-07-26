"""
Usage patterns for the module state API.

This script demonstrates how the module state API would be used in practice.
Note: This won't run until the feature is implemented.
"""


def basic_usage():
    """Basic usage patterns for module state."""
    import module_state

    # Reading state (safe, immutable)
    counter = module_state.get_counter()
    print(f"Current counter: {counter}")

    # Modifying state (requires explicit handling)
    new_value = module_state.increment()
    print(f"After increment: {new_value}")

    # Configuration management
    module_state.set_config("production")
    config = module_state.get_config()
    print(f"Config: {config}")

    # Accessing shared data
    data = module_state.get_data()
    print(f"Shared data: {data}")


def class_integration():
    """Show how classes interact with module state."""
    import module_state

    # Create an instance
    counter = module_state.Counter("metrics")
    print(f"Counter instance: {counter}")

    # Access module state from class methods
    state_value = counter.get_module_state_value()
    print(f"Module state counter from class: {state_value}")


def state_persistence():
    """Demonstrate that state persists across function calls."""
    import module_state

    print(f"Initial counter: {module_state.get_counter()}")

    for i in range(5):
        module_state.increment()

    final = module_state.get_counter()
    print(f"After 5 increments: {final}")
    # State should have been modified


def error_handling():
    """Show how to handle state access errors gracefully."""
    import module_state

    try:
        # Normal access
        value = module_state.get_counter()
        print(f"Successfully accessed counter: {value}")
    except RuntimeError as e:
        print(f"State access error: {e}")


def pattern_counter_service():
    """
    Pattern: Use module state as a simple counter service.

    This shows a realistic use case for module state: maintaining
    application-level counters, metrics, or configuration.
    """
    import module_state

    class MetricsCollector:
        """Collect metrics using module state."""

        def __init__(self, name):
            self.name = name

        def record_event(self):
            """Record that an event occurred."""
            # Each call increments the shared counter
            return module_state.increment()

        def get_total(self):
            """Get the total number of events."""
            return module_state.get_counter()

    # Usage
    metrics = MetricsCollector("app_metrics")
    metrics.record_event()  # counter becomes 1
    metrics.record_event()  # counter becomes 2
    metrics.record_event()  # counter becomes 3

    print(f"Total events recorded: {metrics.get_total()}")


def pattern_config_store():
    """
    Pattern: Use module state as a configuration store.

    This shows how module state can be used to manage
    application configuration that's set once at init time.
    """
    import module_state

    class AppConfig:
        """Application configuration stored in module state."""

        @staticmethod
        def set_environment(env_name):
            """Set the application environment."""
            module_state.set_config(f"environment={env_name}")

        @staticmethod
        def get_environment():
            """Get the current application environment."""
            config = module_state.get_config()
            # Parse environment from config
            if "environment=" in config:
                return config.split("=")[1]
            return None

    # Usage
    AppConfig.set_environment("production")
    print(f"Running in: {AppConfig.get_environment()}")


def pattern_resource_management():
    """
    Pattern: Use module state for resource management.

    This shows how module state could manage shared resources,
    connections, or caches that should persist across calls.
    """
    import module_state

    class DatabaseConnection:
        """Hypothetical: manage a shared database connection in module state."""

        @staticmethod
        def get_shared_data():
            """Get data from the shared resource."""
            # In a real scenario, this would fetch from a
            # database connection stored in module state
            return module_state.get_data()

    # Usage
    data = DatabaseConnection.get_shared_data()
    print(f"Shared resource data: {data}")


if __name__ == "__main__":
    print("=" * 60)
    print("Module State API Usage Patterns")
    print("=" * 60)

    try:
        print("\n1. Basic Usage:")
        print("-" * 60)
        basic_usage()

        print("\n2. Class Integration:")
        print("-" * 60)
        class_integration()

        print("\n3. State Persistence:")
        print("-" * 60)
        state_persistence()

        print("\n4. Error Handling:")
        print("-" * 60)
        error_handling()

        print("\n5. Counter Service Pattern:")
        print("-" * 60)
        pattern_counter_service()

        print("\n6. Configuration Store Pattern:")
        print("-" * 60)
        pattern_config_store()

        print("\n7. Resource Management Pattern:")
        print("-" * 60)
        pattern_resource_management()

    except Exception as e:
        print(f"\nNote: This example won't run until the API is implemented.")
        print(f"Error: {e}")

    print("\n" + "=" * 60)
    print("For implementation status, see:")
    print("  PHASE2_DETAILED_IMPLEMENTATION_PLAN.md")
    print("=" * 60)
