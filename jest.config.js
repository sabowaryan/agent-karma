module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  roots: ['<rootDir>/sdk/src', '<rootDir>/api/src'],
  testMatch: [
    '**/__tests__/**/*.ts',
    '**/?(*.)+(spec|test).ts'
  ],
  transform: {
    '^.+\\.ts$': 'ts-jest',
  },
  collectCoverageFrom: [
    'sdk/src/**/*.ts',
    'api/src/**/*.ts',
    '!**/*.d.ts',
    '!**/node_modules/**',
    '!**/dist/**'
  ],
  coverageDirectory: 'coverage',
  coverageReporters: ['text', 'lcov', 'html'],
  moduleNameMapping: {
    '^@agent-karma/sdk$': '<rootDir>/sdk/src',
    '^@agent-karma/api$': '<rootDir>/api/src'
  },
  setupFilesAfterEnv: ['<rootDir>/jest.setup.js']
};