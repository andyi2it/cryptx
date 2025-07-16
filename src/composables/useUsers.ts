import { ref } from 'vue';

const users = ref([
  'User 1',
  'User 2',
  'User 3',
  'Alice',
  'Bob',
  'Charlie'
]);

export const useUsers = () => {
  const addUser = (name: string) => {
    users.value.push(name);
  };

  const getUsers = () => users.value;

  return {
    users,
    addUser,
    getUsers
  };
};
