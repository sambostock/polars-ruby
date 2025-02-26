module Polars
  # Base class for all Polars data types.
  class DataType
  end

  # Base class for numeric data types.
  class NumericType < DataType
  end

  # Base class for integral data types.
  class IntegralType < NumericType
  end

  # Base class for fractional data types.
  class FractionalType < NumericType
  end

  # Base class for temporal data types.
  class TemporalType < DataType
  end

  # Base class for nested data types.
  class NestedType < DataType
  end

  # 8-bit signed integer type.
  class Int8 < IntegralType
  end

  # 16-bit signed integer type.
  class Int16 < IntegralType
  end

  # 32-bit signed integer type.
  class Int32 < IntegralType
  end

  # 64-bit signed integer type.
  class Int64 < IntegralType
  end

  # 8-bit unsigned integer type.
  class UInt8 < IntegralType
  end

  # 16-bit unsigned integer type.
  class UInt16 < IntegralType
  end

  # 32-bit unsigned integer type.
  class UInt32 < IntegralType
  end

  # 64-bit unsigned integer type.
  class UInt64 < IntegralType
  end

  # 32-bit floating point type.
  class Float32 < FractionalType
  end

  # 64-bit floating point type.
  class Float64 < FractionalType
  end

  # Boolean type.
  class Boolean < DataType
  end

  # UTF-8 encoded string type.
  class Utf8 < DataType
  end

  # Nested list/array type.
  class List < NestedType
    def initialize(inner)
      @inner = Utils.rb_type_to_dtype(inner)
    end
  end

  # Calendar date type.
  class Date < TemporalType
  end

  # Calendar date and time type.
  class Datetime < TemporalType
    attr_reader :tu

    def initialize(time_unit = "us", time_zone = nil)
      @tu = time_unit || "us"
      @time_zone = time_zone
    end
  end

  # Time duration/delta type.
  class Duration < TemporalType
    attr_reader :tu

    def initialize(time_unit = "us")
      @tu = time_unit
    end
  end

  # Time of day type.
  class Time < TemporalType
  end

  # Type for wrapping arbitrary Ruby objects.
  class Object < DataType
  end

  # A categorical encoding of a set of strings.
  class Categorical < DataType
  end

  # Definition of a single field within a `Struct` DataType.
  class Field
    attr_reader :name, :dtype

    def initialize(name, dtype)
      @name = name
      @dtype = Utils.rb_type_to_dtype(dtype)
    end

    def inspect
      class_name = self.class.name
      "#{class_name}(#{@name}: #{@dtype})"
    end
  end

  # Struct composite type.
  class Struct < NestedType
    attr_reader :fields

    def initialize(fields)
      if fields.is_a?(Hash)
        @fields = fields.map { |n, d| Field.new(n, d) }
      else
        @fields = fields
      end
    end

    def inspect
      class_name = self.class.name
      "#{class_name}(#{@fields})"
    end

    def to_schema
      @fields.to_h { |f| [f.name, f.dtype] }
    end
  end

  # Binary type.
  class Binary < DataType
  end

  # Type representing Null / None values.
  class Null < DataType
  end

  # Type representing Datatype values that could not be determined statically.
  class Unknown < DataType
  end
end
