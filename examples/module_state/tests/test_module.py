"""
Test file for module state example.

This demonstrates how the module state API would be used in Python tests.
It won't run until the feature is implemented.
"""

import pytest


class TestModuleState:
    """Tests for module state functionality."""

    def test_counter_initial_value(self):
        """Module state counter starts at 0."""
        import module_state

        assert module_state.get_counter() == 0

    def test_counter_increment(self):
        """Counter increments correctly."""
        import module_state

        assert module_state.increment_counter() == 1
        assert module_state.increment_counter() == 2
        assert module_state.get_counter() == 2

    def test_counter_persists_across_calls(self):
        """Counter value persists across multiple function calls."""
        import module_state

        # Start fresh for determinism (in real tests, use fixtures)
        initial = module_state.get_counter()
        module_state.increment_counter()
        module_state.increment_counter()
        assert module_state.get_counter() == initial + 2

    def test_config_get_set(self):
        """Configuration can be read and written."""
        import module_state

        assert module_state.get_config() == "initialized"

    def test_counter_class_access(self):
        """Counter class can access module state."""
        import module_state

        counter = module_state.Counter("test_counter")
        value = counter.get_shared_counter_value()
        assert isinstance(value, int)
        # Value should match the module-level counter
        assert value == module_state.get_counter()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
