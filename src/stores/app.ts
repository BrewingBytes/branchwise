import { defineStore } from "pinia";
import { DEFAULT_USER } from "../types/user";
import { IGitProject } from "../types/gitProject";

export const useAppStore = defineStore('app', {
    state: () => (
        {
        title: "BranchWise",
        user: DEFAULT_USER,
        projects: [] as IGitProject[],
        isNavbarOpen: false,
        selectedProject: null as IGitProject | null
    }),
    getters: {
        getProjects(): IGitProject[] {
            return this.projects;
        },
        getSelectedProject(): IGitProject | null {
            return this.selectedProject;
        },
    },
    actions: {
        setTitle(title: string) {
            this.title = title;
        },
        addProject(git: IGitProject) {
            this.projects.push(git);
        },
        setProjects(projects: IGitProject[]) {
            this.projects = projects;
        },
        removeProject(git: IGitProject) {
            const index = this.projects.indexOf(git);
            if (index > -1) {
                this.projects.splice(index, 1);
            }
        },
        toggleNavbar() {
            this.isNavbarOpen = !this.isNavbarOpen;
        },
        setCurrentProject(git: IGitProject) {
            this.selectedProject = git;
        }
    }
});
